wit_bindgen::generate!({ path: "wit", world: "command-plugin" });
use craft::plugin::{commands, types::{FieldFailure, PluginError}};
use exports::craft::plugin::entry::Guest;
use serde_json::{json, Value};
struct Rails;
fn invalid(field: &str, reason: &str) -> PluginError {
    PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.into() })
}
fn run(command: &str, params: Value) -> Result<Value, PluginError> {
    let response = commands::run(command, &params.to_string())?;
    let response: Value = serde_json::from_str(&response).map_err(|_| invalid("command", "Invalid host response"))?;
    if let Some(error) = response.get("error").filter(|error| !error.is_null()) {
        return Err(PluginError::CommandFailed(craft::plugin::types::CommandFailure { command: command.into(), reason: error.to_string() }));
    }
    Ok(response)
}
type V3 = [f64; 3];
fn vector(value: &Value) -> Option<V3> {
    let a = value.as_array()?;
    if a.len() != 3 { return None; }
    let v = [a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?];
    v.iter().all(|n| n.is_finite()).then_some(v)
}
fn add(a: V3, b: V3) -> V3 { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn scale(a: V3, s: f64) -> V3 { [a[0]*s,a[1]*s,a[2]*s] }
fn sub(a: V3, b: V3) -> V3 { add(a,scale(b,-1.0)) }
fn dot(a: V3, b: V3) -> f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]] }
fn unit(a: V3) -> Option<V3> { let n=dot(a,a).sqrt(); (n>1e-8).then(|| scale(a,1.0/n)) }
fn selected_edge(pick: &Value) -> Result<(V3,V3), PluginError> {
    let body = pick["body"].as_str().ok_or_else(|| invalid("lines", "Select body edges"))?;
    let point = vector(&pick["point"]).ok_or_else(|| invalid("lines", "Select body edges"))?;
    let result = run("model.edges",json!({"body":body}))?;
    let edges = result["edges"].as_array().ok_or_else(|| invalid("lines", "Could not inspect selected edge"))?;
    let mut best: Option<(f64,V3,V3,bool)> = None;
    for edge in edges {
        let (Some(start),Some(end),Some(mid),Some(len)) = (vector(&edge["start"]),vector(&edge["end"]),vector(&edge["mid"]),edge["length"].as_f64()) else { continue };
        let delta=sub(end,start); let len2=dot(delta,delta);
        if len2 < 1e-16 { continue; }
        let t=(dot(sub(point,start),delta)/len2).clamp(0.0,1.0);
        let distance=dot(sub(point,add(start,scale(delta,t))),sub(point,add(start,scale(delta,t)))).sqrt();
        let straight=(len-len2.sqrt()).abs()<1e-5 && dot(sub(mid,scale(add(start,end),0.5)),sub(mid,scale(add(start,end),0.5))).sqrt()<1e-5;
        if best.is_none_or(|v| distance < v.0) { best=Some((distance,start,end,straight)); }
    }
    let (distance,start,end,straight)=best.ok_or_else(|| invalid("lines", "Could not inspect selected edge"))?;
    if distance > 0.01 || !straight { return Err(invalid("lines", "Select two straight edges away from their endpoints")); }
    Ok((start,end))
}
fn edge_placement(a: (V3,V3), b: (V3,V3)) -> Result<(V3,V3,V3), PluginError> {
    let mut y=unit(sub(a.1,a.0)).ok_or_else(|| invalid("lines", "Select nonzero edges"))?;
    let y2=unit(sub(b.1,b.0)).ok_or_else(|| invalid("lines", "Select nonzero edges"))?;
    if dot(y,y2).abs() < 0.99999 { return Err(invalid("lines", "Select parallel edges")); }
    // Canonical direction makes reversing edge order or endpoints produce the same placement.
    let dominant=(0..3).max_by(|&i,&j| y[i].abs().total_cmp(&y[j].abs())).unwrap();
    if y[dominant] < 0.0 { y=scale(y,-1.0); }
    let a_mid=scale(add(a.0,a.1),0.5); let b_mid=scale(add(b.0,b.1),0.5);
    let mut z=unit(cross(sub(b_mid,a_mid),y)).ok_or_else(|| invalid("lines", "Select two distinct parallel edges"))?;
    let dominant=(0..3).max_by(|&i,&j| z[i].abs().total_cmp(&z[j].abs())).unwrap();
    if z[dominant] < 0.0 { z=scale(z,-1.0); }
    Ok((scale(add(a_mid,b_mid),0.5),y,z))
}
// Width, height, hole pitch, through diameter, counterbore diameter and depth, in mm.
fn rail(size: &str) -> Option<(f64,f64,f64,f64,f64,f64)> {
    match size {
        "MGN7" => Some((7.0,4.8,15.0,2.4,4.0,2.3)),
        "MGN9" => Some((9.0,6.5,20.0,3.5,6.0,3.5)),
        "MGN12" => Some((12.0,8.0,25.0,3.5,6.0,4.5)),
        "MGN15" => Some((15.0,10.0,40.0,4.5,7.5,5.3)),
        _ => None,
    }
}
fn block(size: &str, kind: &str) -> Option<(&'static str, f64, f64)> {
    match (size,kind) {
        ("MGN7","C") => Some((include_str!("../models/MGN7C.step"),0.0,22.5)),
        ("MGN7","H") => Some((include_str!("../models/MGN7H.step"),30.0,30.8)),
        ("MGN9","C") => Some((include_str!("../models/MGN9C.step"),50.0,28.9)),
        ("MGN9","H") => Some((include_str!("../models/MGN9H.step"),0.0,39.9)),
        ("MGN12","C") => Some((include_str!("../models/MGN12C.step"),60.0,34.7)),
        ("MGN12","H") => Some((include_str!("../models/MGN12H.step"),0.0,45.4)),
        ("MGN15","C") => Some((include_str!("../models/MGN15C.step"),70.0,42.1)),
        ("MGN15","H") => Some((include_str!("../models/MGN15H.step"),0.0,58.8)),
        _ => None,
    }
}
impl Guest for Rails {
 fn execute(entry: String, params: String) -> Result<String, PluginError> {
    if entry != "rail" { return Err(invalid("entry", "Unknown entry")); }
    let p: Value = serde_json::from_str(&params).map_err(|_| invalid("params", "Invalid parameters"))?;
    let size = p["type"].as_str().ok_or_else(|| invalid("type", "Choose a rail type"))?;
    let (_w,h,pitch,diam,bore,depth) = rail(size).ok_or_else(|| invalid("type", "Unknown rail type"))?;
    let length = p["length"].as_f64().filter(|v| v.is_finite() && *v >= bore && *v <= 2000.0).ok_or_else(|| invalid("length", "Length must fit a mounting hole and be at most 2000 mm"))?;
    let kind = p.get("block").map_or(Some("none"), Value::as_str).ok_or_else(|| invalid("block", "Choose a block"))?;
    let model = if kind == "none" { None } else { Some(block(size,kind).ok_or_else(|| invalid("block", "Unknown block"))?) };
    if model.is_some_and(|(_,_,l)| length < l) { return Err(invalid("length", "Rail must be at least as long as the block")); }
    let edges = match p.get("lines") {
        None => None,
        Some(Value::Array(picks)) if picks.is_empty() => None,
        Some(Value::Array(picks)) if picks.len() == 2 => Some(edge_placement(selected_edge(&picks[0])?, selected_edge(&picks[1])?)?),
        _ => return Err(invalid("lines", "Select exactly two parallel straight edges")),
    };
    let surface = match p.get("surface") {
        None => None,
        Some(Value::Array(picks)) if picks.is_empty() => None,
        Some(Value::Array(picks)) if picks.len() == 1 && picks[0]["body"].is_string()
            && picks[0]["point"].as_array().is_some_and(|point| point.len() == 3 && point.iter().all(|v| v.as_f64().is_some_and(f64::is_finite))) => Some(&picks[0]),
        _ => return Err(invalid("surface", "Select one planar face")),
    };
    if let Some(surface) = surface {
        let mut item = json!({"type":"face", "body":surface["body"], "point":surface["point"]});
        if let Some(path) = surface.get("path") { item["path"] = path.clone(); }
        let measurement = run("inspect.measure", json!({"items":[item]}))?;
        if measurement["items"][0]["planar"] != true { return Err(invalid("surface", "Select a planar face")); }
    }
    let existing = run("inspect.measure",json!({"names_only":true}))?;
    let bodies = existing["bodies"].as_array().ok_or_else(|| invalid("command", "Could not list bodies"))?;
    let name = (1..).map(|n| format!("{size} Rail {n}")).find(|s| !bodies.iter().any(|b| b["name"].as_str()==Some(s) || b["definition"].as_str()==Some(s))).unwrap();
    let profiles: Value = serde_json::from_str(include_str!("profiles.json")).unwrap();
    let created = run("component.create",json!({"name":name,"parent":0,"activate":true}))?;
    let sketch = format!("{name} profile");
    let extrusion = format!("{name} extrusion");
    let mut features = vec![sketch.clone(), extrusion.clone()];
    run("sketch.create",json!({"plane":"XZ","name":sketch}))?;
    run("sketch.line",json!({"points":profiles[size],"closed":true,"infer":false,"connect":false}))?;
    run("sketch.finish",json!({}))?;
    run("solid.extrude",json!({"sketch":sketch,"distance":length,"operation":"new","name":extrusion,"body_name":name}))?;
    let count = ((length-bore)/pitch).floor() as usize + 1;
    let first = (length - (count-1) as f64*pitch)/2.0;
    for i in 0..count {
        let y = first + i as f64*pitch;
        let hole = format!("{name} hole {}", i+1);
        let counterbore = format!("{name} counterbore {}", i+1);
        run("solid.cylinder",json!({"diameter":diam,"height":h,"base":[0,y,0],"operation":"cut","targets":[name],"name":hole}))?;
        run("solid.cylinder",json!({"diameter":bore,"height":depth,"base":[0,y,h-depth],"operation":"cut","targets":[name],"name":counterbore}))?;
        features.extend([hole,counterbore]);
    }
    if let Some((step, start, block_length)) = model {
        let block_name = format!("{name} Block");
        let names = [block_name.clone(),format!("{block_name} end 1"),format!("{block_name} end 2")];
        run("timeline.add",json!({"feature":{"type":"import","name":block_name,"body_names":names,"file":format!("{size}{kind}.step"),"step":step,"inserted":true}}))?;
        run("component.move_features",json!({"features":[block_name],"component":name}))?;
        let move_name = format!("{block_name} position");
        run("solid.move",json!({"bodies":names,"translate":[0,(length-block_length)/2.0-start,0],"name":move_name}))?;
        features.extend([block_name,move_name]);
    }
    run("timeline.group",json!({"name":name,"features":features}))?;
    run("component.activate",json!({"component":"root"}))?;
    let occurrence = created["occurrence"].as_u64().ok_or_else(|| invalid("command", "Component has no occurrence"))?;
    let aligned = if let Some(surface) = surface {
        Some(run("occurrence.align", json!({"occurrence":occurrence,"from_plane":"XY","to_face":surface["point"],"flip":true,"move_grounded":true}))?)
    } else { None };
    if let Some((center,y,mut z)) = edges {
        if let Some(aligned) = aligned {
            let normal=vector(&aligned["world_transform"][2]).or_else(|| aligned["world_transform"][2].as_array().and_then(|a| vector(&json!([a[0],a[1],a[2]])))).ok_or_else(|| invalid("surface", "Could not determine surface orientation"))?;
            if dot(normal,z).abs() < 0.99999 || dot(sub(center,vector(&surface.unwrap()["point"]).unwrap()),normal).abs() > 0.01 {
                return Err(invalid("lines", "Edges must lie in the selected surface plane"));
            }
            z=normal;
        }
        let x=cross(y,z);
        let origin=sub(center,scale(y,length/2.0));
        let matrix=[[x[0],y[0],z[0],origin[0]],[x[1],y[1],z[1],origin[1]],[x[2],y[2],z[2],origin[2]],[0.0,0.0,0.0,1.0]];
        run("occurrence.move",json!({"occurrence":occurrence,"to_matrix":matrix,"move_grounded":true}))?;
    }
    Ok(json!({"type":size,"length_mm":length,"block":kind}).to_string())
 }
}
#[cfg(target_arch="wasm32")]
export!(Rails);
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn catalog() {
        for s in ["MGN7","MGN9","MGN12","MGN15"] { assert!(rail(s).is_some()); for k in ["C","H"] { assert!(block(s,k).is_some()); } }
        assert!(rail("MGN10").is_none()); assert!(block("MGN7","X").is_none());
    }
    #[test] fn center_parallel_edges() {
        let a=([0.0,0.0,10.0],[0.0,100.0,10.0]);
        let b=([40.0,100.0,10.0],[40.0,0.0,10.0]);
        assert_eq!(edge_placement(a,b).unwrap(),([20.0,50.0,10.0],[0.0,1.0,0.0],[0.0,0.0,1.0]));
        assert_eq!(edge_placement(b,a).unwrap(),edge_placement(a,b).unwrap());
        assert!(edge_placement(a,([0.0,0.0,10.0],[40.0,0.0,10.0])).is_err());
        assert!(edge_placement(a,a).is_err());
    }
}
