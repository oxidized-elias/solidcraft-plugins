# Linear Rails for SolidCraft

Create an MGN7, MGN9, MGN12 or MGN15 rail at a length you choose, with an optional C (short) or H (long) carriage block. Lengths are in millimeters and are limited to 2000 mm. The block sits at the center of the rail, which must be at least as long as the block.

From the repository root, run `./install.sh linear-rails`, then install `dist/linear-rails.zip` from this directory using **Plugins → Install plugin…** in SolidCraft. Open a design and select **Linear Rails** on the ribbon.

To place the rail on a surface, select one planar face under **Mounting surface**. The bottom of the rail starts at the selected point and faces outward. Leave the field empty to create the rail at the origin. SolidCraft chooses the direction along the surface; you can reposition or rotate the component afterward.

To center the rail between two straight parallel body edges, select them under **Center between edges**. The rail runs parallel to the edges and its length is centered at the midpoint between the edges’ midpoints. Pick away from edge endpoints. Select a mounting surface as well to choose its outward side; both edges must lie in that surface plane. Without a surface, the plugin chooses a consistent side of the plane defined by the edges. Placement is fixed and does not create a joint to follow later changes to the surface or edges.

The rail and block bodies are in one component. Their construction features remain editable in one expandable history group.

For each size, the rail section follows the straight-edged end section of the supplied designer's STEP model, including its side grooves and chamfers. Mounting holes are centered along the length at nominal pitch. The block uses the supplied STEP geometry without scaling. The source models approximate the raceway with straight edges and are not a specification for bearing fit or manufacturing. Verify dimensions against your manufacturer's drawings.

[HIWIN's linear-guideway catalog](https://hiwin.co.uk/wp-content/uploads/2017/06/Linear-Guideways.pdf) shows the MGN rail section and nominal rail and mounting dimensions but does not specify the groove contour in sufficient detail to reconstruct it. The groove geometry comes from the supplied reference models.

## Attribution and licensing

The MGN7, MGN9, MGN12 and MGN15 reference assemblies and carriage block geometry were designed by [Dr. Balls](https://grabcad.com/dr.balls-2). Thank you for sharing these models. Their redistribution terms have not been established; obtain any necessary permission before redistributing this plugin or its derived models.

`wit/plugin.wit` is the SolidCraft plugin interface, copied from [SolidCraft](https://github.com/tangency-labs/solidcraft/blob/e8689fd86eac15513db4d91e8b764bf036884d6b/crates/craft-plugin/wit/plugin.wit). It is available under MIT or Apache-2.0; the MIT notice is included in `LICENSES/SolidCraft-MIT.txt`. No license has been selected for the rest of this project.
