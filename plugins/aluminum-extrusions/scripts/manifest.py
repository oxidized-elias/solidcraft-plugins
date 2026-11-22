import json
from pathlib import Path
root=Path(__file__).resolve().parents[1]
profiles=json.loads((root/'src/profiles.json').read_text())
manifest={'id':'aluminum-extrusions','name':'Aluminum Extrusions','version':'0.1.0','description':'Generate extrusions from the included profile library. Lengths are in millimeters.','capabilities':['commands@0.2'],'runtime':{'module':'aluminum-extrusions.wasm','commands':['component.list','sketch.create','sketch.line','sketch.circle.center','sketch.finish','solid.extrude','solid.fillet'],'entries':[{'name':'extrusion','effect':'edit','input_schema':{'type':'object','x-solidcraft-icon':'icons/extrusion.svg','required':['profile','length'],'properties':{'profile':{'type':'string','title':'Profile','default':next(iter(profiles)),'oneOf':[{'const':p,'title':p} for p in profiles]},'length':{'type':'number','title':'Length (mm)','default':100,'exclusiveMinimum':0}}},'slots':[{'kind':'ribbon','name':'Extrusions'}]}],'limits':{'fuel':1000000000,'memory_mib':64,'deadline_ms':30000}}}
(root/'plugin.json').write_text(json.dumps(manifest,indent=2)+'\n')
