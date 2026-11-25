#!/usr/bin/env python3
"""Exercise selected faces against the SolidCraft host."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

cli = sys.argv[1]
root = Path(__file__).resolve().parents[1]
package = root / 'dist/linear-rails'
approval = json.loads(subprocess.check_output([cli, 'exec', 'plugin.inspect', json.dumps({'path': str(package)})]))['approval']
with tempfile.TemporaryDirectory(prefix='linear-rail-surface-') as temp:
    temp = Path(temp)
    for point, axis, expected in [([30, 40, 10], 2, 10), ([100, 40, 5], 0, 100)]:
        commands = [
            {'command': 'solid.box', 'params': {'length': 100, 'width': 100, 'height': 10, 'body_name': 'Plate'}},
            *([{'command': 'component.from_bodies', 'params': {'bodies': ['Plate']}}] if axis == 0 else []),
            {'command': 'plugin.install', 'params': {'path': str(root / 'dist/linear-rails.zip'), 'replace': True}},
            {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/linear-rails'), 'approval': approval, 'entry': 'rail', 'params': {'type': 'MGN12', 'length': 100, 'block': 'C', 'surface': [{'type': 'face', 'body': 'Plate', 'point': point}]}}},
            {'command': 'inspect.measure', 'params': {}},
        ]
        script = temp / 'script.json'
        script.write_text(json.dumps(commands))
        result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, capture_output=True, text=True)
        bodies = json.loads(result.stdout)[-1]['bodies']
        rail = next(b for b in bodies if b['name'] == 'MGN12 Rail 1')
        assert abs(rail['bbox']['min'][axis] - expected) < 1e-5, rail['bbox']
        assert abs(rail['bbox']['max'][axis] - (expected + 8)) < 1e-5, rail['bbox']
        blocks = [b for b in bodies if 'Block' in b['name']]
        assert len(blocks) == 3 and all(b['bbox']['min'][axis] >= expected for b in blocks)
        print(f'pass surface {point}: rail and block face outward')

    for points, surface in [
        ([[0, 30, 10], [40, 70, 10]], None),
        ([[40, 70, 10], [0, 30, 10]], [20, 50, 10]),
        ([[40, 0, 4], [40, 100, 6]], [40, 50, 5]),
    ]:
        params = {'type': 'MGN12', 'length': 40, 'block': 'C', 'lines': [{'type': 'edge', 'body': 'Plate', 'point': p} for p in points]}
        if surface is not None:
            params['surface'] = [{'type': 'face', 'body': 'Plate', 'point': surface}]
        commands = [
            {'command': 'solid.box', 'params': {'length': 40, 'width': 100, 'height': 10, 'body_name': 'Plate'}},
            {'command': 'plugin.install', 'params': {'path': str(root / 'dist/linear-rails.zip'), 'replace': True}},
            {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/linear-rails'), 'approval': approval, 'entry': 'rail', 'params': params}},
            {'command': 'inspect.measure', 'params': {}},
        ]
        script = temp / 'script.json'
        script.write_text(json.dumps(commands))
        result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, capture_output=True, text=True)
        bodies = json.loads(result.stdout)[-1]['bodies']
        rail = next(b for b in bodies if b['name'] == 'MGN12 Rail 1')
        lo, hi = rail['bbox']['min'], rail['bbox']['max']
        if points[0][2] == 10:
            assert all(abs(a-b)<1e-5 for a,b in zip(lo,[14,30,10])), rail['bbox']
            assert all(abs(a-b)<1e-5 for a,b in zip(hi,[26,70,18])), rail['bbox']
        else:
            assert all(abs(a-b)<1e-5 for a,b in zip(lo,[40,44,-15])), rail['bbox']
            assert all(abs(a-b)<1e-5 for a,b in zip(hi,[48,56,25])), rail['bbox']
        print(f'pass centered edges {points}, surface={surface}')
