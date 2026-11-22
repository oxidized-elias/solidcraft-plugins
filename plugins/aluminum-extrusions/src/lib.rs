wit_bindgen::generate!({ path: "wit", world: "command-plugin" });
use craft::plugin::{commands, types::{FieldFailure, PluginError}};
use exports::craft::plugin::entry::Guest;
use serde_json::{json, Value};

struct Extrusions;
fn invalid(field: &str, reason: &str) -> PluginError {
    PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.into() })
}
fn run(command: &str, params: Value) -> Result<Value, PluginError> {
    let reply = commands::run(command, &params.to_string())?;
    let reply: Value = serde_json::from_str(&reply).map_err(|_| invalid("command", "Invalid host response"))?;
    if let Some(error) = reply.get("error") {
        return Err(PluginError::CommandFailed(craft::plugin::types::CommandFailure { command: command.into(), reason: error.to_string() }));
    }
    Ok(reply)
}

fn unique_name(base: &str, components: &Value) -> Result<String, PluginError> {
    let components = components["components"].as_array().ok_or_else(|| invalid("command", "Invalid component list"))?;
    let names: Vec<&str> = components.iter().flat_map(|c| ["features", "sketches", "bodies"].into_iter().flat_map(move |k| c[k].as_array().into_iter().flatten().filter_map(Value::as_str))).collect();
    let mut name = base.to_string();
    let mut suffix = 1;
    while names.contains(&name.as_str()) {
        name = format!("{base} ({suffix})");
        suffix += 1;
    }
    Ok(name)
}
impl Guest for Extrusions {
    fn execute(entry: String, params: String) -> Result<String, PluginError> {
        if entry != "extrusion" { return Err(invalid("entry", "Unknown entry")); }
        let input: Value = serde_json::from_str(&params).map_err(|_| invalid("params", "Invalid parameters"))?;
        let length = input["length"].as_f64().filter(|v| v.is_finite() && *v > 0.0).ok_or_else(|| invalid("length", "Length must be greater than zero"))?;
        let name = input["profile"].as_str().ok_or_else(|| invalid("profile", "Choose a profile"))?;
        let profiles: Value = serde_json::from_str(include_str!("profiles.json")).unwrap();
        let p = profiles.get(name).ok_or_else(|| invalid("profile", "Unknown profile"))?;
        let label = unique_name(&format!("{name} - {length}mm"), &run("component.list", json!({}))?)?;
        run("sketch.create", json!({"plane":"XY","name":label}))?;
        for points in p["loops"].as_array().unwrap() {
            run("sketch.line", json!({"points":points,"closed":true,"infer":false,"connect":false}))?;
        }
        for circle in p["circles"].as_array().unwrap() {
            run("sketch.circle.center", json!({"center":[circle[0],circle[1]],"diameter":circle[2],"connect":false}))?;
        }
        run("sketch.finish", json!({}))?;
        run("solid.extrude", json!({"sketch":label,"distance":length,"operation":"new","name":label,"body_name":label}))?;
        for fillet in p["fillets"].as_array().unwrap() {
            let edges: Vec<Value> = fillet["edges"].as_array().unwrap().iter().map(|p| json!([p[0],p[1],length/2.0])).collect();
            run("solid.fillet", json!({"edges":edges,"radius":fillet["radius"]}))?;
        }
        Ok(json!({"profile":name,"length_mm":length}).to_string())
    }
}
export!(Extrusions);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_avoid_existing_features_sketches_and_bodies() {
        let components = json!({"components":[{"features":["2040 - 330mm"],"sketches":["2040 - 330mm (1)"],"bodies":[]}]});
        assert_eq!(unique_name("2040 - 330mm", &components).unwrap(), "2040 - 330mm (2)");
        assert_eq!(unique_name("2020 - 10.5mm", &components).unwrap(), "2020 - 10.5mm");
    }
}
