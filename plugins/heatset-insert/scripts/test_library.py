#!/usr/bin/env python3
"""Exercise actual plugin installation and imports using the embedded CNC Kitchen library."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

cli = sys.argv[1]
root = Path(__file__).resolve().parent.parent
package = root / "heatset-insert"
archive = root / "heatset-insert.zip"
inspect = subprocess.run([cli, "exec", "plugin.inspect", json.dumps({"path": str(package)})], check=True, capture_output=True, text=True)
approval = json.loads(inspect.stdout)["approval"]
cases = [("M2", "standard", 0, 3), ("M2.5", "standard", 0, 4),
         ("M3", "standard", 7, 7), ("M3", "short", 0, 3), ("M3", "voron", 0, 4),
         ("M4", "standard", 0, 8.1), ("M4", "short", 0, 4),
         ("M5", "standard", 0, 9.5), ("M5", "short", 0, 5.8),
         ("M6", "standard", 0, 12.7), ("M8", "standard", 0, 12.7)]
with tempfile.TemporaryDirectory(prefix="heatset-test-") as temp:
    for size, variant, length, expected in cases:
        plugins = Path(temp) / "plugins"
        script = [
            {"command": "plugin.install", "params": {"path": str(archive), "replace": True}},
            {"command": "plugin.run", "params": {"path": str(plugins / "heatset-insert"), "approval": approval, "entry": "insert",
             "params": {"size": size, "variant": variant, "length": length}}},
            {"command": "inspect.measure", "params": {}}]
        script_path = Path(temp) / "case.json"
        script_path.write_text(json.dumps(script))
        result = subprocess.run([cli, "--plugins-dir", str(plugins), "run", str(script_path), "--results"], check=True, capture_output=True, text=True)
        reply = json.loads(result.stdout)
        body = reply[2]["bodies"][0]
        bbox = body["bbox"]
        assert reply[1]["result"]["length_mm"] == expected
        assert abs((bbox["max"][2] - bbox["min"][2]) - expected) < 0.001
        assert body["faces"] > 50
        print(f"pass {size} {variant}: {expected} mm")
