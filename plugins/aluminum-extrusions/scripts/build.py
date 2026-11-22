#!/usr/bin/env python3
"""Rebuild the profile catalog and installable SolidCraft plugin ZIP."""
import shutil, subprocess, sys, zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
def run(*args): subprocess.run(args,cwd=root,check=True)
run(sys.executable,'scripts/extract.py')
run(sys.executable,'scripts/manifest.py')
run('cargo','build','--release','--locked','--target','wasm32-unknown-unknown')
package=root/'dist/aluminum-extrusions'
package.mkdir(parents=True,exist_ok=True)
run('wasm-tools','component','new','target/wasm32-unknown-unknown/release/aluminum_extrusions.wasm','-o',str(package/'aluminum-extrusions.wasm'))
run('wasm-tools','validate',str(package/'aluminum-extrusions.wasm'))
shutil.copy(root/'plugin.json',package/'plugin.json')
shutil.copytree(root/'icons',package/'icons',dirs_exist_ok=True)
cli=shutil.which('solidcraft-cli')
if not cli:
    candidate=Path.home()/'.local/share/flatpak/app/io.tangencylabs.solidcraft/current/active/files/bin/solidcraft-cli'
    if candidate.is_file(): cli=str(candidate)
if not cli: sys.exit('Install solidcraft-cli to check the plugin before packaging.')
shutil.copytree(root/'tests',package/'tests',dirs_exist_ok=True)
run(cli,'plugin','check',str(package))
run(cli,'plugin','test',str(package),'--require-cases','--out',str(root/'dist/test-results'))
with zipfile.ZipFile(root/'dist/aluminum-extrusions.zip','w',zipfile.ZIP_DEFLATED) as z:
    for name in ['plugin.json','aluminum-extrusions.wasm']: z.write(package/name,name)
    z.write(root/'README.md','README.md')
    z.write(root/'LICENSES/SolidCraft-MIT.txt','LICENSES/SolidCraft-MIT.txt')
    z.write(root/'icons/extrusion.svg','icons/extrusion.svg')
print('Built dist/aluminum-extrusions.zip')
