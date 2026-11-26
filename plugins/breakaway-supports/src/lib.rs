wit_bindgen::generate!({ path: "wit", world: "command-plugin" });
use craft::plugin::{commands, control, types::{FieldFailure, PluginError, CommandFailure}};
use exports::craft::plugin::entry::Guest;
use serde_json::{json, Value};
struct Supports;
fn invalid(field: &str, reason: &str) -> PluginError { PluginError::InvalidInput(FieldFailure { field: field.into(), reason: reason.into() }) }
fn run(command: &str, args: Value) -> Result<Value, PluginError> {
    let reply = commands::run(command, &args.to_string())?;
    let result: Value = serde_json::from_str(&reply).map_err(|_| invalid("command", "Invalid host response"))?;
    if let Some(error) = result.get("error").filter(|e| !e.is_null()) { return Err(PluginError::CommandFailed(CommandFailure { command: command.into(), reason: error.to_string() })); }
    Ok(result)
}
fn vec3(v: &Value) -> Option<[f64;3]> { let a=v.as_array()?; if a.len()!=3 {return None} let r=[a[0].as_f64()?,a[1].as_f64()?,a[2].as_f64()?]; r.iter().all(|v|v.is_finite()).then_some(r) }
fn number(p: &Value, key: &str, default: f64, min: f64, max: f64) -> Result<f64, PluginError> {
    let v=match p.get(key) { None => default, Some(v) => v.as_f64().ok_or_else(||invalid(key,"Enter a number"))? };
    if !v.is_finite() || v < min || v > max { return Err(invalid(key,"Value is outside the allowed range")); } Ok(v)
}
fn ray(x:f64,y:f64,z:f64,dir:f64)->Value { json!({"origin":[x,y,z],"dir":[0,0,dir]}) }
fn hits(rays:Vec<Value>)->Result<Vec<Value>,PluginError> {
    let result=run("view.pick",json!({"rays":rays}))?;
    result["hits"].as_array().cloned().ok_or_else(||invalid("faces","Could not inspect the selected geometry"))
}
fn grid(min:f64,max:f64,spacing:f64)->Vec<f64> { let n=((max-min)/spacing).ceil() as usize; (0..n).map(|i|min+(i as f64+0.5)*spacing).filter(|x|*x<max).collect() }
impl Guest for Supports {
 fn execute(entry:String, params:String)->Result<String,PluginError> {
    if entry!="generate" {return Err(invalid("entry","Unknown entry"))}
    let p:Value=serde_json::from_str(&params).map_err(|_|invalid("params","Invalid parameters"))?;
    let faces=p["faces"].as_array().filter(|a|!a.is_empty()).ok_or_else(||invalid("faces","Select at least one downward-facing horizontal face"))?;
    let spacing=number(&p,"spacing",5.0,2.0,50.0)?;
    let thickness=number(&p,"diameter",1.0,0.4,5.0)?;
    let wall=number(&p,"wall_thickness",0.8,0.4,5.0)?;
    let bridge=number(&p,"bridge_thickness",1.0,1.0,10.0)?;
    let clearance=number(&p,"side_clearance",1.0,0.2,5.0)?;
    let gap=number(&p,"gap",0.2,0.05,1.0)?;
    let dot=number(&p,"dot_diameter",0.6,0.2,5.0)?;
    let on_part=match p.get("on_part") {None=>false,Some(v)=>v.as_bool().ok_or_else(||invalid("on_part","Choose true or false"))?};
    if dot >= thickness {return Err(invalid("dot_diameter","Dot diameter must be smaller than wall thickness"))}
    if thickness>=spacing {return Err(invalid("diameter","Wall thickness must be smaller than spacing"))}
    control::progress(0.02,"Inspecting model and locating its base") ;
    let measure=run("inspect.measure",json!({}))?;
    let bodies=measure["bodies"].as_array().ok_or_else(||invalid("faces","Could not inspect bodies"))?;
    let bed=bodies.iter().filter_map(|b|vec3(&b["bbox"]["min"])).map(|v|v[2]).fold(f64::INFINITY,f64::min);
    if !bed.is_finite() {return Err(invalid("faces","Could not determine the lowest point of the model"))}
    let mut wrap=false;
    let mut rounded=None;
    let mut center_hole=None;
    let mut selected_bounds=None;
    let mut profiled=false;
    let mut pillars:Vec<(f64,f64,f64,f64,bool)>=Vec::new();
    for (face_number,face) in faces.iter().enumerate() {
        control::progress(0.06+0.19*face_number as f32/faces.len() as f32,&format!("Finding contact points on face {} of {}",face_number+1,faces.len()));
        let start=pillars.len();
        let body=face["body"].as_str().ok_or_else(||invalid("faces","Select body faces"))?;
        let point=vec3(&face["point"]).ok_or_else(||invalid("faces","Select body faces"))?;
        let bounds=bodies.iter().find(|b|b["name"].as_str()==Some(body)).ok_or_else(||invalid("faces","Selected body could not be found"))?;
        let lo=vec3(&bounds["bbox"]["min"]).ok_or_else(||invalid("faces","Could not inspect body bounds"))?;
        let hi=vec3(&bounds["bbox"]["max"]).ok_or_else(||invalid("faces","Could not inspect body bounds"))?;
        selected_bounds=Some((lo,hi));
        let selected=hits(vec![ray(point[0],point[1],point[2]-0.001,1.0)])?;
        let selected=&selected[0];
        if selected["body"].as_str()!=Some(body) || vec3(&selected["normal"]).is_none_or(|n| n[2]>-0.99999) || (vec3(&selected["point"]).unwrap_or([0.0;3])[2]-point[2]).abs()>0.01 {return Err(invalid("faces","Select a downward-facing horizontal face"))}
        let index=selected["face"].clone();
        if faces.len()==1 {
            let geometry=run("model.faces",json!({"body":body}))?;
            if let Some(fs)=geometry["faces"].as_array() {
                let cylinders:Vec<(f64,f64,f64,f64)>=fs.iter().filter_map(|f| {
                    let c=&f["cylinder"]; let o=vec3(&c["origin"])?; let axis=vec3(&c["axis"])?; let r=c["radius"].as_f64()?; let fp=vec3(&f["point"])?;
                    let d=((point[0]-o[0]).powi(2)+(point[1]-o[1]).powi(2)).sqrt();
                    (axis[2].abs()>0.9999 && fp[2]>=point[2]-0.01 && (r-d).abs()<=spacing && r>wall*2.0).then_some((o[0],o[1],r,d))
                }).collect();
                if let Some(nearest)=cylinders.iter().min_by(|a,b|a.3.total_cmp(&b.3)) {
                    rounded=cylinders.iter().filter(|c|(c.0-nearest.0).abs()<0.01 && (c.1-nearest.1).abs()<0.01).max_by(|a,b|a.2.total_cmp(&b.2)).map(|c|(c.0,c.1,c.2));
                }
                if let Some((cx,cy,outer))=rounded {
                    center_hole=fs.iter().filter_map(|f| {
                        let c=&f["cylinder"]; let o=vec3(&c["origin"])?; let axis=vec3(&c["axis"])?; let r=c["radius"].as_f64()?;
                        (axis[2].abs()>0.9999 && (o[0]-cx).abs()<0.01 && (o[1]-cy).abs()<0.01 && r<outer-0.01 && r>1.0).then_some(r)
                    }).min_by(f64::total_cmp);
                }
            }
        }
        let xs=grid(lo[0],hi[0],spacing); let ys=grid(lo[1],hi[1],spacing);
        if xs.len().saturating_mul(ys.len())>400 {return Err(invalid("spacing","Too many grid points; increase spacing"))}
        let mut points=Vec::new();
        for x in xs { for y in &ys { points.push((x,*y)); } }
        if points.is_empty() {return Err(invalid("faces","Selected face is too small for this spacing"))}
        let rays=points.iter().map(|(x,y)|ray(*x,*y,point[2]-0.001,1.0)).collect();
        let top_hits=hits(rays)?;
        let mut candidates=Vec::new();
        for ((x,y),hit) in points.into_iter().zip(top_hits) {
            if hit["body"].as_str()==Some(body) && hit["face"]==index && !pillars.iter().any(|v|(v.0-x).abs()<0.001&&(v.1-y).abs()<0.001&&(v.2+v.3-(point[2]-gap)).abs()<0.001) { candidates.push((x,y)); }
        }
        if candidates.is_empty(){return Err(invalid("faces","No support samples fit one of the selected faces; reduce spacing or remove duplicate selections"))}
        let r=thickness/2.0;
        // shortcut: five rays can miss narrow obstacles, upgrade to swept-volume collision checks when available.
        let offsets=[(0.0,0.0),(r,0.0),(-r,0.0),(0.0,r),(0.0,-r)];
        let mut rays=Vec::new();
        for (x,y) in &candidates {for (dx,dy) in offsets {rays.push(ray(x+dx,y+dy,point[2]-gap,-1.0));}}
        let mut lower=Vec::new();
        for chunk in rays.chunks(1000) {lower.extend(hits(chunk.to_vec())?);}
        for ((x,y),sample) in candidates.into_iter().zip(lower.chunks(5)) {
            let mut base=bed; let mut blocked=false;
            for hit in sample { if let Some(pt)=vec3(&hit["point"]) {if pt[2]>=bed-0.0001 {blocked=true; base=base.max(pt[2]+gap);}} }
            if blocked && !on_part {wrap=true; base=bed; blocked=false;}
            if blocked && vec3(&sample[0]["point"]).is_none_or(|pt| (pt[2]+gap-base).abs()>0.001) {continue}
            let height=point[2]-gap-base;
            if height>0.1 {pillars.push((x,y,base,height,blocked));}
        }
        if pillars.len()==start {return Err(invalid("faces","One of the selected faces has no clear support area"))}
    }
    if pillars.is_empty() {return Err(invalid("faces","No supports fit; check the build plate, selected faces and spacing"))}
    if pillars.len()>200 {return Err(invalid("spacing","Too many supports; increase spacing"))}
    let base=pillars[0].2; let height=pillars[0].3; let blocked=pillars[0].4;
    if height < bridge + 1.0 || pillars.iter().any(|p|(p.2-base).abs()>0.01 || (p.3-height).abs()>0.01 || p.4!=blocked) {
        return Err(invalid("faces","A single solid support requires selected areas at the same heights and a level base"));
    }
    let rounded=if wrap {rounded} else {None};
    let margin=if wrap {spacing/2.0+wall+0.5} else {thickness/2.0};
    let mut min_x=pillars.iter().map(|p|p.0).fold(f64::INFINITY,f64::min)-margin;
    let mut max_x=pillars.iter().map(|p|p.0).fold(f64::NEG_INFINITY,f64::max)+margin;
    let mut min_y=pillars.iter().map(|p|p.1).fold(f64::INFINITY,f64::min)-margin;
    let mut max_y=pillars.iter().map(|p|p.1).fold(f64::NEG_INFINITY,f64::max)+margin;
    if let Some((cx,cy,r))=rounded {
        let radius=r+wall+clearance+0.5;
        min_x=cx-radius; max_x=cx+radius; min_y=cy-radius; max_y=cy+radius;
    }
    if max_x-min_x <= 2.0*wall+0.1 || max_y-min_y <= 2.0*wall+0.1 {return Err(invalid("wall_thickness","Support footprint is too narrow for a hollow interior; reduce wall thickness or contact spacing"))}
    let nx=((max_x-min_x)/(spacing/4.0)).ceil() as usize;
    let ny=((max_y-min_y)/(spacing/4.0)).ceil() as usize;
    if nx.saturating_mul(ny)>4000 {return Err(invalid("spacing","Solid support footprint is too large; increase spacing"))}
    let mut rays=Vec::new();
    for ix in 0..=nx { for iy in 0..=ny {
        rays.push(ray(min_x+(max_x-min_x)*ix as f64/nx as f64,min_y+(max_y-min_y)*iy as f64/ny as f64,base+height,-1.0));
    }}
    control::progress(0.27,"Checking the support footprint and lower surfaces");
    let mut lower_surfaces=Vec::new();
    if wrap {
        for chunk in rays.chunks(1000) { for hit in hits(chunk.to_vec())? {
            if let (Some(pt),Some(normal))=(vec3(&hit["point"]),vec3(&hit["normal"])) {
                if normal[2]>0.999 && pt[2]>bed+0.001 && pt[2]+gap<base+height-0.01 {lower_surfaces.push(pt);}
            }
        }}
    }
    for chunk in rays.chunks(1000) {if wrap {break} for hit in hits(chunk.to_vec())? {
        let lower=vec3(&hit["point"]);
        if if blocked {lower.is_none_or(|p|(p[2]-(base-gap)).abs()>0.01)} else {lower.is_some_and(|p|p[2]>=bed-0.001)} {
            return Err(invalid("faces","The solid support footprint intersects the part or has an uneven base"));
        }
    }}
    control::progress(0.35,"Building the hollow support shell");
    let mut features=vec!["Breakaway support".to_string(),"Support hollow".to_string()];
    if let Some((cx,cy,r))=rounded {
        let bottom=r+wall+clearance+0.5; let top=r+wall+clearance;
        run("solid.cone",json!({"radius":bottom,"top_radius":top,"height":height,"base":[cx,cy,base],"operation":"new","name":"Breakaway support","body_name":"Breakaway support"}))?;
        run("solid.cone",json!({"radius":bottom-wall,"top_radius":top-wall+wall*bridge/height,"height":height-bridge,"base":[cx,cy,base],"operation":"cut","targets":["Breakaway support"],"name":"Support hollow"}))?;
        if let Some(inner)=center_hole {
            run("solid.cylinder",json!({"radius":inner-0.2,"height":height+0.02,"base":[cx,cy,base-0.01],"operation":"cut","targets":["Breakaway support"],"name":"Support center opening"}))?;
            features.push("Support center opening".into());
        }
        let panel_width=r; let panel_height=(height-bridge).min(r*0.8); let panel_z=base+(height-bridge-panel_height)/2.0;
        run("solid.box",json!({"length":panel_width,"width":wall*2.0,"height":panel_height,"corner":[cx-panel_width/2.0,cy-bottom, panel_z],"operation":"join","targets":["Breakaway support"],"name":"Support X panel"}))?;
        features.push("Support X panel".into());
        if let Some((_,hi))=selected_bounds {
            if hi[1]-cy>r*2.0 {
                let limit=cy+r-wall-clearance;
                run("solid.box",json!({"length":max_x-min_x+2.0,"width":max_y-limit+1.0,"height":height+2.0,"corner":[min_x-1.0,limit,base-1.0],"operation":"cut","targets":["Breakaway support"],"name":"Support wall clearance"}))?;
                features.push("Support wall clearance".into());
                let opening=vec![[limit+0.1,base+height*0.35],[limit-r.min(8.0)*0.45,base+height-bridge-0.5],[limit+0.1,base+height+0.1]];
                run("sketch.create",json!({"plane":{"origin":[cx,0,0],"x_dir":[0,1,0],"y_dir":[0,0,1]},"name":"Support angled opening"}))?;
                run("sketch.line",json!({"points":opening,"closed":true,"infer":false,"connect":false}))?;
                run("sketch.finish",json!({}))?;
                run("solid.extrude",json!({"sketch":"Support angled opening","distance":2.0*(bottom+1.0),"direction":"symmetric","operation":"cut","targets":["Breakaway support"],"name":"Support opening"}))?;
                features.extend(["Support angled opening".into(),"Support opening".into()]);
                profiled=true;
                pillars.retain(|p|p.1+dot/2.0<limit);
                if pillars.is_empty() {return Err(invalid("faces","No contact points remain after leaving space for the wall"))}
            }
        }
    } else {
        run("solid.box",json!({"length":max_x-min_x,"width":max_y-min_y,"height":height,"corner":[min_x,min_y,base],"operation":"new","name":"Breakaway support","body_name":"Breakaway support"}))?;
        run("solid.box",json!({"length":max_x-min_x-2.0*wall,"width":max_y-min_y-2.0*wall,"height":height-bridge+0.01,"corner":[min_x+wall,min_y+wall,base-0.01],"operation":"cut","targets":["Breakaway support"],"name":"Support hollow"}))?;
    }
    if wrap {
        control::progress(0.43,"Cutting the model shape from the support");
        let tools:Vec<&str>=bodies.iter().filter(|b| {
            let Some(lo)=vec3(&b["bbox"]["min"]) else {return true};
            let Some(hi)=vec3(&b["bbox"]["max"]) else {return true};
            hi[0]>=min_x-clearance && lo[0]<=max_x+clearance && hi[1]>=min_y-clearance && lo[1]<=max_y+clearance && hi[2]>=base-gap && lo[2]<=base+height+gap
        }).filter_map(|b|b["name"].as_str()).collect();
        run("solid.combine",json!({"target":"Breakaway support","tools":tools,"operation":"cut","keep_tools":true,"name":"Support clearance"}))?;
        features.push("Support clearance".into());
        // shortcut: translated copies approximate clearance; use an exact offset when available.
        let directions=[(clearance,0.0),(-clearance,0.0),(0.0,clearance),(0.0,-clearance),(clearance,clearance),(clearance,-clearance),(-clearance,clearance),(-clearance,-clearance)];
        let mut offsets=vec![(0.0,0.0,gap)];
        for (x,y) in directions {offsets.push((x,y,0.0)); if x==0.0 || y==0.0 {offsets.push((x,y,gap));}}
        // shortcut: profiled openings use geometric clearance, verify unfamiliar part shapes.
        if profiled { offsets.truncate(1); }
        for (i,(dx,dy,dz)) in offsets.iter().enumerate() {
            for (j,body) in tools.iter().enumerate() {
                let done=(i*tools.len()+j) as f32;
                let total=(offsets.len()*tools.len()) as f32;
                control::progress(0.45+0.30*done/total,&format!("Adding side clearance: cut {} of {}",i*tools.len()+j+1,offsets.len()*tools.len()));
                let copy=format!("Support clearance tool {i}-{j}");
                let moved=format!("Support clearance copy {i}-{j}");
                let cut=format!("Support clearance cut {i}-{j}");
                run("solid.move",json!({"bodies":[body],"copy":true,"translate":[dx,dy,dz],"body_name":copy,"name":moved}))?;
                run("solid.combine",json!({"target":"Breakaway support","tools":[copy],"operation":"cut","name":cut}))?;
                features.extend([moved,cut]);
            }
        }
    }
    control::progress(0.76,"Finding lower surfaces that need contact dots");
    let mut intermediate:Vec<(f64,Vec<(f64,f64)>)>=Vec::new();
    for chunk in lower_surfaces.chunks(1000) {
        let rays=chunk.iter().map(|p|ray(p[0],p[1],p[2]+0.001,1.0)).collect();
        for (pt,hit) in chunk.iter().zip(hits(rays)?) {
            if hit["body"].as_str()!=Some("Breakaway support") || vec3(&hit["point"]).is_none_or(|v| (v[2]-(pt[2]+gap)).abs()>0.02) {continue}
            if let Some((_,dots))=intermediate.iter_mut().find(|(z,_)|(z-pt[2]).abs()<0.001) {dots.push((pt[0],pt[1]));}
            else {intermediate.push((pt[2],vec![(pt[0],pt[1])]));}
        }
    }
    if intermediate.iter().map(|(_,v)|v.len()).sum::<usize>()>200 {return Err(invalid("spacing","Too many intermediate contacts; increase spacing"))}
    let mut dot_sets=vec![("Support top contacts",base+height,pillars.iter().map(|p|(p.0,p.1)).collect::<Vec<_>>(),dot)];
    let intermediate_names:Vec<String>=(0..intermediate.len()).map(|i|format!("Support intermediate contacts {}",i+1)).collect();
    for ((z,points),name) in intermediate.into_iter().zip(intermediate_names.iter()) {dot_sets.push((name.as_str(),z,points,dot));}
    if blocked {dot_sets.push(("Support bottom contacts",base-gap,vec![(min_x+wall/2.0,min_y+wall/2.0),(max_x-wall/2.0,min_y+wall/2.0),(min_x+wall/2.0,max_y-wall/2.0),(max_x-wall/2.0,max_y-wall/2.0)],dot.min(wall)));}
    let dot_count=dot_sets.len();
    for (set_index,(name,z,points,dot_size)) in dot_sets.into_iter().enumerate() {
        control::progress(0.80+0.10*set_index as f32/dot_count as f32,&format!("Creating contact dots: group {} of {}",set_index+1,dot_count));
        let sketch=format!("{name} sketch");
        run("sketch.create",json!({"plane":{"origin":[0,0,z],"x_dir":[1,0,0],"y_dir":[0,1,0]},"name":sketch}))?;
        for (x,y) in points {run("sketch.circle.center",json!({"center":[x,y],"diameter":dot_size,"connect":false}))?;}
        run("sketch.finish",json!({}))?;
        run("solid.extrude",json!({"sketch":sketch,"distance":gap,"operation":"join","targets":["Breakaway support"],"name":name}))?;
        features.extend([sketch,name.into()]);
    }
    control::progress(0.91,"Adding the X mark");
    let size=if let Some((_,_,r))=rounded { (r*0.5).min((height-bridge)*0.4) } else { ((max_x-min_x)*0.5).min((height-bridge)*0.5) };
    let w=size*0.22;
    let cx=(min_x+max_x)/2.0; let cz=base+(height-bridge)/2.0; let h=size/2.0;
    let points=vec![[cx-h,cz-h+w],[cx-h+w,cz-h],[cx,cz-w],[cx+h-w,cz-h],[cx+h,cz-h+w],[cx+w,cz],[cx+h,cz+h-w],[cx+h-w,cz+h],[cx,cz+w],[cx-h+w,cz+h],[cx-h,cz+h-w],[cx-w,cz]];
    let depth=(wall/2.0).min(0.4);
    run("sketch.create",json!({"plane":{"origin":[0,min_y+depth/2.0,0],"x_dir":[1,0,0],"y_dir":[0,0,1]},"name":"Support X"}))?;
    run("sketch.line",json!({"points":points,"closed":true,"infer":false,"connect":false}))?;
    run("sketch.finish",json!({}))?;
    run("solid.extrude",json!({"sketch":"Support X","distance":depth,"direction":"symmetric","operation":"cut","targets":["Breakaway support"],"name":"Support X mark"}))?;
    control::progress(0.96,"Checking the completed support and grouping its history");
    let final_body=run("inspect.measure",json!({}))?;
    if final_body["bodies"].as_array().is_none_or(|b|b.len()!=bodies.len()+1 || b.iter().filter(|v|v["name"]=="Breakaway support" && v["closed"]==true).count()!=1) {return Err(invalid("faces","The support could not be made into one closed body"))}
    features.extend(["Support X".into(),"Support X mark".into()]);
    run("timeline.group",json!({"name":"Breakaway support","features":features}))?;
    control::progress(1.0,"Support complete");
    Ok(json!({"supports":1,"x_panel":true,"rounded":rounded.is_some(),"profiled":profiled}).to_string())
 }
}
#[cfg(target_arch="wasm32")]
export!(Supports);
#[cfg(test)]
mod tests { use super::*; #[test] fn grid_points_are_inside_bounds() {assert_eq!(grid(0.0,11.0,5.0),vec![2.5,7.5]); assert!(grid(0.0,0.0,5.0).is_empty());} }
