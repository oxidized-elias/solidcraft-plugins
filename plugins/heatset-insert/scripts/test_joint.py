#!/usr/bin/env python3
"""Verify that selected holes receive a rigidly jointed insert in the real host."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

cli = sys.argv[1]
root = Path(__file__).resolve().parent.parent
approval = json.loads(subprocess.check_output([cli, "exec", "plugin.inspect", json.dumps({"path": str(root / "heatset-insert")})]))["approval"]
with tempfile.TemporaryDirectory(prefix="heatset-joint-") as temp:
    temp = Path(temp)
    for size, length, flip in [("M3", 5.7, False), ("M3", 5.7, True), ("M3.5", 7.1, False), ("M3.5", 7.1, True), ("M3", 8.0, True)]:
        script = [
            {"command": "solid.box", "params": {"length": 20, "width": 20, "height": 10, "corner": [0, 0, 0], "body_name": "Plate"}},
            {"command": "solid.cylinder", "params": {"diameter": 4, "height": 10, "base": [10, 10, 0], "operation": "cut", "targets": ["Plate"]}},
            {"command": "component.from_bodies", "params": {"bodies": ["Plate"]}},
            {"command": "plugin.install", "params": {"path": str(root / "heatset-insert.zip"), "replace": True}},
            {"command": "plugin.run", "params": {"path": str(temp / "plugins/heatset-insert"), "approval": approval, "entry": "insert", "params": {"size": size, "length": length, "flip": flip, "hole": [{"type": "face", "body": "Plate", "point": [12, 10, 9]}]}}},
            {"command": "joint.list", "params": {}},
            {"command": "inspect.measure", "params": {}},
        ]
        (temp / "script.json").write_text(json.dumps(script))
        result = subprocess.run([cli, "--plugins-dir", str(temp / "plugins"), "run", str(temp / "script.json"), "--results"], check=True, capture_output=True, text=True)
        reply = json.loads(result.stdout)
        assert len(reply[5]["joints"]) == 1 and reply[5]["joints"][0]["type"] == "rigid"
        assert not reply[5]["errors"] and not reply[5]["conflicts"]
        bounds = reply[6]["bodies"][1]["bbox"]
        assert abs((bounds["min"][0] + bounds["max"][0]) / 2 - 10) < 0.01
        assert abs((bounds["min"][1] + bounds["max"][1]) / 2 - 10) < 0.01
        assert abs(bounds["min"][2] - (10 - length)) < 0.01
        assert abs(bounds["max"][2] - 10) < 0.01
        assert reply[5]["joints"][0]["flip"] == flip
        print(f"pass {size}: rigidly placed in hole, flip={flip}")

    for size, length in [("M3", 5.7), ("M3.5", 7.1)]:
        script = [
            {"command": "solid.box", "params": {"length": 30, "width": 20, "height": 10, "corner": [0, 0, 0], "body_name": "Plate"}},
            *[{"command": "solid.cylinder", "params": {"diameter": 4, "height": 10, "base": [x, 10, 0], "operation": "cut", "targets": ["Plate"]}} for x in (8, 22)],
            {"command": "component.from_bodies", "params": {"bodies": ["Plate"]}},
            {"command": "plugin.install", "params": {"path": str(root / "heatset-insert.zip"), "replace": True}},
            {"command": "plugin.run", "params": {"path": str(temp / "plugins/heatset-insert"), "approval": approval, "entry": "insert", "params": {"size": size, "flip": True, "hole": [{"type": "face", "body": "Plate", "point": [x + 2, 10, 9]} for x in (8, 22)]}}},
            {"command": "joint.list", "params": {}},
            {"command": "inspect.measure", "params": {}},
        ]
        (temp / "script.json").write_text(json.dumps(script))
        result = subprocess.run([cli, "--plugins-dir", str(temp / "plugins"), "run", str(temp / "script.json"), "--results"], check=True, capture_output=True, text=True)
        reply = json.loads(result.stdout)
        assert reply[5]["result"]["count"] == 2
        assert len(reply[6]["joints"]) == 2 and not reply[6]["errors"] and not reply[6]["conflicts"]
        bodies = reply[7]["bodies"][1:]
        assert len(bodies) == 2
        for body, x in zip(bodies, (8, 22)):
            bounds = body["bbox"]
            assert abs((bounds["min"][0] + bounds["max"][0]) / 2 - x) < 0.01
            assert abs((bounds["min"][1] + bounds["max"][1]) / 2 - 10) < 0.01
            assert abs(bounds["min"][2] - (10 - length)) < 0.01 and abs(bounds["max"][2] - 10) < 0.01
        print(f"pass {size}: two inserts placed and jointed")
