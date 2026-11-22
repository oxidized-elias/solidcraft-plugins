#!/usr/bin/env python3
"""Convert supported SolidCraft extrusion designs into plugin profile data."""
import json, math, sys, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def extract(path):
    with zipfile.ZipFile(path) as z:
        doc = json.loads(z.read('document.json'))
    features = [f for f in doc['features'] if not f.get('suppressed')]
    sketches = [f for f in features if f['type'] == 'sketch']
    extrudes = [f for f in features if f['type'] == 'extrude']
    if len(sketches) != 1 or len(extrudes) != 1 or any(f['type'] not in ('sketch', 'extrude', 'fillet') for f in features):
        raise ValueError(f'{path.name}: expected one sketch, one extrusion and optional fillets')
    sketch = sketches[0]['sketch']
    points = [p['pos'] for p in sketch['points']]
    edges, circles = [], []
    for c in sketch['curves']:
        if c.get('construction'): continue
        kind = c['type']
        if kind == 'circle':
            circles.append([*points[c['c']], 2*c['r']])
        elif kind == 'line':
            edges.append([points[c['a']], points[c['b']]])
        elif kind == 'arc':
            a,b,center = points[c['a']],points[c['b']],points[c['c']]
            start = math.atan2(a[1]-center[1],a[0]-center[0]); end = math.atan2(b[1]-center[1],b[0]-center[0])
            sweep = (end-start) % (2*math.pi)
            if sweep > math.pi: sweep -= 2*math.pi
            r = math.dist(a,center); n = max(1,math.ceil(abs(sweep)/math.radians(3)))
            # shortcut: arcs use 3-degree chords; upgrade to native arc commands for exact curves.
            edges.append([a]+[[center[0]+r*math.cos(start+sweep*i/n),center[1]+r*math.sin(start+sweep*i/n)] for i in range(1,n)]+[b])
        else: raise ValueError(f'{path.name}: unsupported curve {kind}')
    loops=[]
    while edges:
        loop=edges.pop()
        while math.dist(loop[-1],loop[0]) > 1e-5:
            for i,e in enumerate(edges):
                if math.dist(loop[-1],e[0]) < 1e-5: loop += e[1:]; edges.pop(i); break
                if math.dist(loop[-1],e[-1]) < 1e-5: loop += e[-2::-1]; edges.pop(i); break
            else: raise ValueError(f'{path.name}: open or ambiguous contour near {loop[-1]}')
        loops.append(loop[:-1])
    fillets=[]
    for f in features:
        if f['type']=='fillet':
            fillets.append({'radius':float(f['radius']), 'edges':[[x,y] for x,y,z in f['edges']]})
    return {'loops':loops,'circles':circles,'fillets':fillets}

def main():
    result={p.stem:extract(p) for p in sorted((ROOT/'profiles').glob('*.sc'))}
    if not result: raise ValueError('No profiles found')
    (ROOT/'src/profiles.json').write_text(json.dumps(result,separators=(',',':')))
    print('Extracted:', ', '.join(result))
if __name__ == '__main__':
    try: main()
    except (ValueError, KeyError, zipfile.BadZipFile) as e: sys.exit(str(e))
