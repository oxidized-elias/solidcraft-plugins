# Breakaway Supports for SolidCraft

The plugin makes one hollow support body beneath selected horizontal faces. For a circular overhang it recognizes, it makes a tapered round shell. If it detects a long adjoining wall, it cuts an angled opening toward it. Other cases use a rectangular shell.

Install the ZIP through **Plugins → Install plugin…**, then choose **Breakaway Supports**. Orient the part for printing with Z upward and select the faces. The support starts at the lowest point among model bodies.

SolidCraft's operations panel shows a progress bar and a note for the current stage, including individual clearance cuts. Progress is an estimate: a complex modeling operation can take time without updating the bar. Preview behavior depends on the host.

The support is open at the bottom, with a recessed X on its side and a continuous solid upper section. **Shell wall thickness** controls the side walls. The construction and X sketch appear together in one expandable history group. **Minimum sacrificial top thickness** sets the solid material below the contact interface; the default 1 mm is five 0.2 mm layers. Configure the slicer wall and infill settings to print the solid upper section as intended, and check the preview.

Small contact dots connect the support to the selected faces. Set **Interface layer height** to your slicer layer height and **Contact dot diameter** to a printable size. With **Allow supports to start on the part** enabled, dots also connect the support's bottom to a level surface on the part. Export the support and part together, and inspect the sliced layers to confirm that the contacts print and connect.

The footprint must be wide enough for the chosen wall thickness. The selected areas must have the same height and a level base; otherwise the plugin refuses to make the single solid. The footprint comes from sampled contact positions or a detected circular surface, so it can extend beyond the selected faces.

When the model lies under the selected face, the support wraps around it and cuts the model shape out of the support. The plugin shapes the opening and may subtract shifted copies of the model to approximate **Side clearance**. It also adds dots where sampled horizontal surfaces lie directly beneath the shell. Clearance is approximate, and these cuts do not guarantee printable slopes. Other collision checks are sampled and can miss narrow features between samples. Inspect the sliced layers before printing.

## Settings

| Setting | Meaning |
|---|---|
| Faces to support | One or more downward-facing horizontal faces at the same height. |
| Allow supports to start on the part | Lets the support start on a level model surface; otherwise it starts on the build plate. |
| Contact spacing | Distance between the contact dots. Smaller values create more contacts and can fit smaller faces, but may make removal harder. |
| Footprint margin | Total width added around the outermost contact centers. Each side extends by half this value. |
| Shell wall thickness | Thickness of the hollow support's side walls. The footprint must be wider than twice this value. |
| Minimum sacrificial top thickness | Thickness of the solid roof below the dots. At 0.2 mm per layer, 1 mm is five layers. |
| Side clearance | Approximate lateral clearance around model walls in the wrapping case. Larger values may weaken or disconnect the support. |
| Contact dot diameter | Width of the small contact points. Smaller dots may break away more easily but provide less support. |
| Interface layer height | Height of the contact dots, not an empty gap. Set it to the slicer layer height. |

The default 0.6 mm dots and 0.2 mm interface height are starting values, not a verified recipe. No geometry guarantees breakaway: the result depends on nozzle, layer height, material, slicer settings and orientation. Print a small test and adjust the dot diameter or spacing before using it on a finished part.

This version supports only flat horizontal downward-facing faces. Round-shell detection is limited to vertical cylindrical geometry and does not reproduce every contour of the supplied example. Small faces may need closer contact spacing. Selected faces and generated supports do not update automatically when the part changes.

Build with `./install.sh breakaway-supports`. The plugin uses SolidCraft's MIT-licensed plugin interface; see `LICENSES/SolidCraft-MIT.txt`.

## Test fixture and attribution

`tests/hinge.sc` contains the hinge geometry supplied for the support-generation tests; its source design identifies itself as E3NG all-in-one. Its redistribution terms have not been established. Confirm the original design's license and any required attribution before publishing the repository. This test fixture is not included in the plugin ZIP.
