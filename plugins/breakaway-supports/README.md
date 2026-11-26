# Breakaway Supports for SolidCraft

Generate one hollow support body beneath selected horizontal faces. For a recognized circular overhang above the model, the support uses a tapered round shell; where a long adjoining wall is detected, it adds an angled opening toward the wall. Other cases use a rectangular shell. Install the ZIP through **Plugins → Install plugin…**, then choose **Breakaway Supports**. Orient the part for printing with Z upward, then select the faces. The support starts at the lowest point among model bodies.

When you run the plugin, SolidCraft's operations panel shows a progress bar and a note for the current stage, including individual clearance cuts. Progress is an estimate; a complex modeling operation can take time without updating the bar. Preview behavior depends on the host.

The support is open on the bottom, has a recessed X on its side, and has a continuous solid upper section. Set **Shell wall thickness** to control the side walls. Its construction and X sketch appear together in one expandable history group. Set **Minimum sacrificial top thickness** to the amount of solid material you want below the contact interface; the default 1 mm is five 0.2 mm layers. The upper section is solid geometry, so configure slicer wall and infill settings to make it print as intended and check the preview.

Small contact dots connect the support to the selected faces. Set **Interface layer height** to your slicer layer height and **Contact dot diameter** to a printable size. With **Allow supports to start on the part** enabled, dots also connect the support's bottom to a level surface on the part. Export the support and part together, and inspect the sliced layers to confirm that the contacts print and connect.

The footprint must be wide enough for the chosen wall thickness. The selected areas must have the same height and a level base; otherwise the plugin refuses to make the single solid. The footprint is derived from sampled contact positions or a detected circular surface, so it can extend beyond the selected faces. When the model lies under the selected face, the support wraps around it and cuts the model shape out of the support. The plugin shapes the opening and may subtract shifted copies of the model to approximate **Side clearance** and adds dots where sampled horizontal surfaces lie directly beneath the shell. Clearance is approximate, and these cuts do not guarantee printable slopes; inspect the sliced layers before printing. Other collision checks are sampled and can miss narrow features between samples.


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
