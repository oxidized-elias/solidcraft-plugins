import json
import math
import subprocess
import sys
import tempfile
from pathlib import Path

cli = sys.argv[1]
root = Path(__file__).resolve().parents[1]
package = root / 'dist/breakaway-supports'
approval = json.loads(subprocess.check_output([cli, 'exec', 'plugin.inspect', json.dumps({'path': str(package)})]))['approval']
with tempfile.TemporaryDirectory(prefix='supports-test-') as directory:
    temp = Path(directory)
    for on_part in (False, True):
        params = {'faces': [{'body': 'Overhang', 'point': [2, 2, 10]}], 'on_part': on_part, 'spacing': 5, 'diameter': 1, 'gap': 0.2, 'dot_diameter': 0.6, 'bridge_thickness': 2, 'wall_thickness': 0.8}
        steps = [
            {'command': 'solid.box', 'params': {'length': 10, 'width': 10, 'height': 2, 'corner': [0, 0, 10], 'body_name': 'Overhang'}},
            {'command': 'solid.box', 'params': {'length': 10, 'width': 10, 'height': 3, 'corner': ([0, 0, 0] if on_part else [30, 0, 0]), 'body_name': 'Obstacle'}},
            {'command': 'plugin.install', 'params': {'path': str(root / 'dist/breakaway-supports.zip'), 'replace': True}},
            {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/breakaway-supports'), 'approval': approval, 'entry': 'generate', 'params': params}},
            {'command': 'inspect.measure', 'params': {}},
            {'command': 'view.pick', 'params': {'rays': [
                {'origin': [5, 0, (5.5 if on_part else 3.9)], 'dir': [0, 1, 0]},
                {'origin': [5, 0, (7.7 if on_part else 6.1)], 'dir': [0, 1, 0]},
                {'origin': [5, 5, (3.3 if on_part else 0.1)], 'dir': [0, 0, 1]}
            ]}},
            {'command': 'timeline.inspect', 'params': {'group': (11 if on_part else 9)}}

        ]
        script = temp / 'script.json'
        script.write_text(json.dumps(steps))
        result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, stdout=subprocess.PIPE, text=True)
        results = json.loads(result.stdout)
        bodies = results[-3]['bodies']
        assert results[-2]['hits'][0]['body'] == 'Breakaway support', results[-2]
        assert results[-2]['hits'][1]['body'] == 'Breakaway support', results[-2]
        assert results[-2]['hits'][0]['depth'] > results[-2]['hits'][1]['depth'] + 0.3, results[-2]
        cavity = results[-2]['hits'][2]
        assert cavity['body'] == 'Breakaway support' and abs(cavity['point'][2] - 7.8) < 1e-5, cavity
        supports = [b for b in bodies if b['name'] == 'Breakaway support']
        assert len(supports) == 1, supports
        support = supports[0]
        assert support['closed'], support
        assert abs(support['bbox']['max'][2] - 10.0) < 1e-5, support
        assert abs(support['bbox']['min'][2] - (3.0 if on_part else 0.0)) < 1e-5, support
        assert support['volume_mm3'] < 6*6*9.8 - 100, support
        assert len(results[-1]['features']) == (8 if on_part else 6), results[-1]
        print(f'pass: on_part={on_part}, {len(supports)} supports')

    params = {'faces': [{'body': 'First', 'point': [2, 2, 10]}, {'body': 'Second', 'point': [22, 2, 10]}], 'spacing': 5}
    steps = [
        {'command': 'solid.box', 'params': {'length': 10, 'width': 10, 'height': 2, 'corner': [0, 0, 10], 'body_name': 'First'}},
        {'command': 'solid.box', 'params': {'length': 10, 'width': 10, 'height': 2, 'corner': [20, 0, 10], 'body_name': 'Second'}},
        {'command': 'solid.box', 'params': {'length': 2, 'width': 2, 'height': 2, 'corner': [40, 0, 0], 'body_name': 'Reference'}},
        {'command': 'plugin.install', 'params': {'path': str(root / 'dist/breakaway-supports.zip'), 'replace': True}},
        {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/breakaway-supports'), 'approval': approval, 'entry': 'generate', 'params': params}},
        {'command': 'inspect.measure', 'params': {}}
    ]
    script = temp / 'multi.json'
    script.write_text(json.dumps(steps))
    result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, stdout=subprocess.PIPE, text=True)
    supports = [b for b in json.loads(result.stdout)[-1]['bodies'] if b['name'] == 'Breakaway support']
    assert len(supports) == 1, supports
    assert supports[0]['bbox']['max'][0] > 27 and supports[0]['bbox']['min'][0] < 3
    print('pass: multiple selected faces')

    steps = [
        {'command': 'solid.cylinder', 'params': {'diameter': 20, 'height': 4, 'base': [0, 0, 0], 'name': 'lower', 'body_name': 'Part'}},
        {'command': 'solid.cylinder', 'params': {'diameter': 20, 'height': 4, 'base': [0, 0, 20], 'name': 'upper', 'body_name': 'Upper', 'operation': 'new'}},
        {'command': 'solid.box', 'params': {'length': 2, 'width': 4, 'height': 20, 'corner': [8, -2, 4], 'name': 'spine', 'operation': 'join', 'targets': ['Part']}},
        {'command': 'solid.combine', 'params': {'target': 'Part', 'tools': ['Upper'], 'operation': 'join'}},
        {'command': 'solid.cylinder', 'params': {'diameter': 12, 'height': 24, 'base': [0, 0, 0], 'operation': 'cut', 'targets': ['Part'], 'name': 'bore'}},
        {'command': 'inspect.measure', 'params': {}},
        {'command': 'plugin.install', 'params': {'path': str(root / 'dist/breakaway-supports.zip'), 'replace': True}},
        {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/breakaway-supports'), 'approval': approval, 'entry': 'generate', 'params': {'faces': [{'body': 'Part', 'point': [0, 8, 20]}]}}},
        {'command': 'inspect.measure', 'params': {}},
        {'command': 'view.pick', 'params': {'rays': [{'origin': [0, 0, 19], 'dir': [0, 0, 1]}]}}
    ]
    script = temp / 'ring.json'
    script.write_text(json.dumps(steps))
    result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, stdout=subprocess.PIPE, text=True)
    outputs = json.loads(result.stdout)
    assert outputs[-3]['result']['rounded'] is True, outputs[-3]
    assert outputs[-1]['hits'][0] is None, outputs[-1]
    bodies = outputs[-2]['bodies']
    before = next(b for b in outputs[-5]['bodies'] if b['name'] == 'Part')
    after = next(b for b in bodies if b['name'] == 'Part')
    assert abs(before['volume_mm3'] - after['volume_mm3']) < 1e-5
    supports = [b for b in bodies if b['name'] == 'Breakaway support']
    assert len(bodies) == 2 and len(supports) == 1 and supports[0]['closed'], bodies
    assert abs(supports[0]['bbox']['min'][2]) < 1e-5, supports
    print('pass: ring over lower part, one closed support on the model base')

    steps.insert(5, {'command': 'solid.box', 'params': {'length': 5, 'width': 8, 'height': 8, 'corner': [9, -4, 0], 'body_name': 'Shelf'}})
    steps.append({'command': 'document.inspect', 'params': {}})
    steps.append({'command': 'sketch.inspect', 'params': {'sketch': 'Support intermediate contacts 1 sketch'}})
    script = temp / 'shelf.json'
    script.write_text(json.dumps(steps))
    result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, stdout=subprocess.PIPE, text=True)
    outputs = json.loads(result.stdout)
    doc = outputs[-2]
    assert any(x['name'] == 'Support intermediate contacts 1' and x['error'] is None for x in doc['timeline']), doc['timeline']
    assert any(x['name'] == 'Support intermediate contacts 1 sketch' and x['profiles'] > 0 for x in doc['sketches']), doc['sketches']
    print('pass: intermediate shelf receives breakaway contact dots')

    steps = steps[:-2]
    steps[5] = {'command': 'solid.box', 'params': {'length': 5, 'width': 5, 'height': 8, 'corner': [5, 0, 0], 'body_name': 'Shelf'}}
    steps.append({'command': 'view.pick', 'params': {'rays': [{'origin': [10.001, 5.001, 6], 'dir': [0.707106781, 0.707106781, 0]}]}})
    script = temp / 'corner.json'
    script.write_text(json.dumps(steps))
    result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--results'], check=True, stdout=subprocess.PIPE, text=True)
    hit = json.loads(result.stdout)[-1]['hits'][0]
    assert hit is None or hit['depth'] >= 0.99, hit
    print('pass: at least 1 mm clearance at an outside corner')

    steps = [
        {'command': 'plugin.install', 'params': {'path': str(root / 'dist/breakaway-supports.zip'), 'replace': True}},
        {'command': 'plugin.run', 'params': {'path': str(temp / 'plugins/breakaway-supports'), 'approval': approval, 'entry': 'generate', 'params': {'faces': [{'body': 'enclosure_v2_hinge_base', 'point': [-4.099134936144825, 5.685478586164138, 33.44557351927905]}]}}},
        {'command': 'inspect.measure', 'params': {}}
    ]
    steps.append({'command': 'view.pick', 'params': {'rays': [{'origin': [-6.112578, 2.837415, 4], 'dir': [-1, 0, 0]}]}})
    script = temp / 'hinge.json'
    script.write_text(json.dumps(steps))
    result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--in', str(root / 'tests/hinge.sc'), '--results'], check=True, stdout=subprocess.PIPE, text=True, timeout=60)
    outputs = json.loads(result.stdout)
    bodies = outputs[-2]['bodies']
    side_hit = outputs[-1]['hits'][0]
    assert side_hit is None or side_hit['depth'] >= 0.99, side_hit
    supports = [b for b in bodies if b['name'] == 'Breakaway support']
    assert outputs[-3]['result']['rounded'] is True and outputs[-3]['result']['profiled'] is True and len(supports) == 1 and supports[0]['closed'], bodies
    print('pass: supplied hinge generates one closed round support')

    baseline = supports[0]['bbox']
    for x, y in [(-3, 0), (-5, 8), (11, 2), (5, 12)]:
        steps[1]['params']['params']['faces'][0]['point'] = [x, y, 33.445573519279]
        script.write_text(json.dumps(steps))
        result = subprocess.run([cli, '--plugins-dir', str(temp / 'plugins'), 'run', str(script), '--in', str(root / 'tests/hinge.sc'), '--results'], check=True, stdout=subprocess.PIPE, text=True, timeout=60)
        outputs = json.loads(result.stdout)
        support = next(b for b in outputs[-2]['bodies'] if b['name'] == 'Breakaway support')
        assert outputs[-3]['result']['rounded'] is True and outputs[-3]['result']['profiled'] is True and support['closed'], support
        assert all(abs(a-b) < 1e-4 for key in ('min', 'max') for a, b in zip(support['bbox'][key], baseline[key])), support['bbox']
    print('pass: same hinge face generates the same support at four other pick locations')
