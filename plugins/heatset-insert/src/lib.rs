wit_bindgen::generate!({ path: "wit", world: "command-plugin" });

use craft::plugin::{commands, types::{FieldFailure, PluginError}};
use exports::craft::plugin::entry::Guest;
use serde_json::{json, Value};

struct Insert;

fn dimensions(size: &str) -> Option<(f64, f64, f64)> {
    // Heat-Sert K (outside diameter), nominal thread and recommended hole, in mm.
    match size {
        "M2" => Some((3.5, 2.0, 3.2)),
        "M2.5" => Some((4.4, 2.5, 4.0)),
        "M3" => Some((4.4, 3.0, 4.0)),
        "M3.5" => Some((5.2, 3.5, 4.8)),
        "M4" => Some((6.1, 4.0, 5.6)),
        "M5" => Some((6.9, 5.0, 6.4)),
        "M6" => Some((8.5, 6.0, 8.0)),
        "M8" => Some((10.0, 8.0, 9.6)),
        _ => None,
    }
}

fn invalid(field: &str, reason: impl ToString) -> PluginError {
    PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.to_string() })
}

fn place(body: &str, hole: Option<&Value>, flip: bool, length: f64) -> Result<(), PluginError> {
    let result = commands::run("component.from_bodies", &json!({"bodies": [body]}).to_string())?;
    if let Some(hole) = hole {
        let result: Value = serde_json::from_str(&result).map_err(|e| invalid("result", e))?;
        let component = &result["components"][0]["component"];
        let list = commands::run("component.list", "{}")?;
        let list: Value = serde_json::from_str(&list).map_err(|e| invalid("result", e))?;
        let occurrence = list["components"].as_array().and_then(|items| items.iter().find(|item| &item["id"] == component)).and_then(|item| item["occurrences"][0]["id"].as_u64()).ok_or_else(|| invalid("result", "Insert component has no occurrence"))?;
        let mut target = json!({"body": hole["body"], "circle": hole["point"]});
        if let Some(path) = hole.get("path") { target["path"] = path.clone(); }
        commands::run("joint.create", &json!({"type": "rigid", "a": target, "b": {"occurrence": occurrence, "point": [0, 0, if flip { length } else { 0.0 }], "z": [0, 0, 1]}, "flip": flip}).to_string())?;
    }
    Ok(())
}

fn source_model(size: &str, variant: &str) -> Option<(&'static str, f64)> {
    match (size, variant) {
        ("M2", "standard" | "short") => Some(("M2_Short.step", 3.0)),
        ("M2.5", "standard" | "short") => Some(("M2p5_Short.step", 4.0)),
        ("M3", "standard") => Some(("M3_Standard.step", 5.7)),
        ("M3", "short") => Some(("M3_Short.step", 3.0)),
        ("M3", "voron") => Some(("M3_VORON.step", 4.0)),
        ("M4", "standard") => Some(("M4_Standard.step", 8.1)),
        ("M4", "short") => Some(("M4_Short.step", 4.0)),
        ("M5", "standard") => Some(("M5_Standard.step", 9.5)),
        ("M5", "short") => Some(("M5_Short.step", 5.8)),
        ("M6", "standard") => Some(("M6_Standard.step", 12.7)),
        ("M8", "standard") => Some(("M8_Standard.step", 12.7)),
        _ => None,
    }
}

impl Guest for Insert {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        if entry != "insert" { return Err(invalid("entry", "Unknown entry")); }
        let params: Value = serde_json::from_str(&params).map_err(|e| invalid("params", e))?;
        let flip = params.get("flip").map_or(Some(false), Value::as_bool).ok_or_else(|| invalid("flip", "Expected true or false"))?;
        if let Some(Value::Array(picks)) = params.get("hole") {
            if picks.len() > 1 {
                if picks.iter().any(|p| !p["body"].is_string() || !p["point"].as_array().is_some_and(|point| point.len() == 3 && point.iter().all(|v| v.as_f64().is_some_and(f64::is_finite)))) {
                    return Err(invalid("hole", "Select cylindrical hole faces"));
                }
                let mut single = params.clone();
                let mut result = String::new();
                for pick in picks {
                    single["hole"] = json!([pick]);
                    result = Self::execute(entry.clone(), single.to_string())?;
                }
                let mut result: Value = serde_json::from_str(&result).map_err(|e| invalid("result", e))?;
                result["count"] = json!(picks.len());
                return Ok(result.to_string());
            }
        }
        let selected_hole = match params.get("hole") {
            None => None,
            Some(Value::Array(picks)) if picks.is_empty() => None,
            Some(Value::Array(picks)) if picks.len() == 1 && picks[0]["body"].is_string() && picks[0]["point"].as_array().is_some_and(|p| p.len() == 3 && p.iter().all(|v| v.as_f64().is_some_and(f64::is_finite))) => Some(&picks[0]),
            _ => return Err(invalid("hole", "Select one cylindrical hole face")),
        };
        let size = params.get("size").map_or(Some("M3"), Value::as_str).ok_or_else(|| invalid("size", "Expected a thread size"))?;
        let (outer, thread, hole) = dimensions(size).ok_or_else(|| invalid("size", "Unsupported thread size"))?;
        let variant = params.get("variant").map_or(Some("standard"), Value::as_str).ok_or_else(|| invalid("variant", "Expected a variant"))?;
        let length = params.get("length").map_or(Some(0.0), Value::as_f64).filter(|v| v.is_finite() && (*v == 0.0 || *v >= 0.1)).ok_or_else(|| invalid("length", "Use 0 for the original length, or at least 0.1 mm"))?;
        if size != "M3.5" {
            let (file, original_length) = source_model(size, variant).ok_or_else(|| invalid("variant", "This size and variant is not in the STEP library"))?;
            let step = match file {
                "M2_Short.step" => include_str!("../models/M2_Short.step"),
                "M2p5_Short.step" => include_str!("../models/M2p5_Short.step"),
                "M3_Standard.step" => include_str!("../models/M3_Standard.step"),
                "M3_Short.step" => include_str!("../models/M3_Short.step"),
                "M3_VORON.step" => include_str!("../models/M3_VORON.step"),
                "M4_Standard.step" => include_str!("../models/M4_Standard.step"),
                "M4_Short.step" => include_str!("../models/M4_Short.step"),
                "M5_Standard.step" => include_str!("../models/M5_Standard.step"),
                "M5_Short.step" => include_str!("../models/M5_Short.step"),
                "M6_Standard.step" => include_str!("../models/M6_Standard.step"),
                "M8_Standard.step" => include_str!("../models/M8_Standard.step"),
                _ => return Err(invalid("size", "Missing bundled model")),
            };
            let existing = commands::run("inspect.measure", &json!({"names_only": true}).to_string())?;
            let existing: Value = serde_json::from_str(&existing).map_err(|e| invalid("result", e))?;
            let bodies = existing["bodies"].as_array().ok_or_else(|| invalid("result", "Could not list bodies"))?;
            let body = (1..).map(|i| format!("Heat Set Insert {i}")).find(|name| !bodies.iter().any(|b| b["name"].as_str() == Some(name) || b["definition"].as_str() == Some(name))).unwrap();
            let feature = json!({"type":"import", "name":body, "body_names":[body], "file":file, "step":step, "inserted":true});
            commands::run("timeline.add", &json!({"feature":feature}).to_string())?;
            if length > 0.0 && (length - original_length).abs() > 1e-9 {
                commands::run("solid.scale", &json!({"bodies": [body], "factors": [1, 1, length / original_length], "origin": [0, 0, 0]}).to_string())?;
            }
            place(&body, selected_hole, flip, if length == 0.0 { original_length } else { length })?;
            return Ok(json!({"size": size, "variant": variant, "source_file": file, "length_mm": if length == 0.0 { original_length } else { length }, "length_scaled": length > 0.0 && (length - original_length).abs() > 1e-9}).to_string());
        }
        if variant != "standard" { return Err(invalid("variant", "M3.5 only has a generated standard model")); }
        let length = if length == 0.0 { 7.1 } else { length };
        let result = commands::run("solid.cylinder", &json!({"diameter": format!("{outer} mm"), "height": format!("{length} mm"), "operation": "new_body"}).to_string())?;
        let result: Value = serde_json::from_str(&result).map_err(|e| invalid("result", e))?;
        let body = result["created_bodies"][0].as_str().ok_or_else(|| invalid("result", "Cylinder did not report its body"))?;
        commands::run("solid.cylinder", &json!({"diameter": format!("{thread} mm"), "height": format!("{length} mm"), "operation": "cut", "targets": [body]}).to_string())?;
        // shortcut: catalog omits groove dimensions; replace these proportions if a detailed drawing is available.
        let groove = (outer - hole).max(0.2) / 2.0;
        for z in [length * 0.3, length * 0.7] {
            commands::run("solid.torus", &json!({"major": format!("{} mm", outer / 2.0), "minor": format!("{} mm", groove.min(length * 0.1)), "center": [0, 0, z], "operation": "cut", "targets": [body]}).to_string())?;
        }
        for i in 0..12 {
            let angle = i as f64 * std::f64::consts::TAU / 12.0;
            commands::run("solid.cylinder", &json!({"diameter": format!("{} mm", groove * 0.9), "height": format!("{length} mm"), "base": [outer / 2.0 * angle.cos(), outer / 2.0 * angle.sin(), 0], "operation": "cut", "targets": [body]}).to_string())?;
        }
        commands::run("solid.thread", &json!({"faces": [{"body": body, "point": [thread / 2.0, 0, length * 0.1]}], "designation": size, "modeled": true}).to_string())?;
        place(body, selected_hole, flip, length)?;
        Ok(json!({"size": size, "length_mm": length, "source": "generated approximation"}).to_string())
    }
}
#[cfg(target_arch = "wasm32")]
export!(Insert);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_variants() {
        assert_eq!(source_model("M3", "short"), Some(("M3_Short.step", 3.0)));
        assert_eq!(source_model("M8", "standard"), Some(("M8_Standard.step", 12.7)));
        assert_eq!(source_model("M8", "short"), None);
        assert_eq!(dimensions("M7"), None);
    }
}
