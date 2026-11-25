#!/usr/bin/env python3
"""Check the saved host-test designs retain their component and history group."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

cli = sys.argv[1]
root = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory() as temp:
    temp = Path(temp)
    script = temp / 'inspect.json'
    script.write_text(json.dumps({'commands': [{'command': 'component.list', 'params': {}}]}))
    for size in ('MGN7', 'MGN9', 'MGN12', 'MGN15'):
        for block in ('none', 'C', 'H'):
            design = root / 'dist/test-results' / f'{size}-{block}.sc'
            components = json.loads(subprocess.check_output([cli, 'run', str(script), '--in', str(design), '--results']))[0]['components']
            assert len(components) == 2 and not components[0]['bodies']
            assert len(components[1]['bodies']) == (1 if block == 'none' else 4)
            out = temp / 'design.json'
            subprocess.run([cli, 'run', '--in', str(design), '--out', str(out)], check=True, stdout=subprocess.DEVNULL)
            saved = json.loads(out.read_text())
            groups = saved['timeline_groups']
            assert len(groups) == 1 and len(groups[0]['features']) == len(saved['features'])
print('Component and history group checks passed for all 12 cases')
