# Heat-Set Insert Generator for SolidCraft

Add the bundled CNC Kitchen STEP models to a SolidCraft design. The plugin supports M2, M2.5, M3, M4, M5, M6 and M8 inserts, with standard and short variants where available, plus M3 Voron. The supplied library has no M3.5 model, so the plugin generates an approximation for that size.

## Credit and source

The STEP models are **Threaded Inserts / Heat Set Inserts by CNC Kitchen**, by CNC Kitchen, from [Printables](https://www.printables.com/model/482734-threaded-inserts-heat-set-inserts-by-cnc-kitchen/files). Thank you to CNC Kitchen for making the CAD models available. The bundled STEP models are compacted copies of the supplied files. Their numeric values were rounded to eight significant digits to fit SolidCraft’s command-size limit. Check the source page for the models' license and terms before redistributing them.

## Install and use

Download `heatset-insert.zip` from [GitHub Releases](https://github.com/oxidized-elias/solidcraft-plugins/releases) and select it in SolidCraft under **Plugins → Install plugin…**. To build it yourself:

1. Build the ZIP with `./install.sh heatset-insert` from the repository root. In SolidCraft choose **Plugins → Install plugin…** and select `dist/heatset-insert.zip` in this plugin's directory. The STEP models are embedded in the plugin, so you do not need a library folder or file access grant.
2. Run **Heat Set Inserts → insert** from the Plugins menu or the Plugins ribbon tab.
3. Select the size and variant. Leave length at **0** for the source length, or enter a length in mm to scale it along Z. To place inserts in holes, select one or more cylindrical walls near the ends where the inserts should sit. Each hole must belong to a component; use **Create Components from Bodies** on its body if necessary. The plugin creates an insert component and rigid joint for each selected hole, using the same settings for all. Enable **Flip insert end** to swap which end faces out while keeping it seated. If you do not select a hole, the plugin places one insert at the origin.

Changing the length scales the entire model in Z, **including the pitch of its modeled threads**. The scaled model can be useful for visualization, but its thread is no longer a standard metric thread. Use length 0 when thread fit matters, or select a provided short/standard variant. M3.5 is generated with an actual metric thread at the requested length, but its external details are approximate.

| Size | Variants | Original length(s), mm |
|---|---|---|
| M2 | standard or short (same short model) | 3.0 |
| M2.5 | standard or short (same short model) | 4.0 |
| M3 | standard, short, voron | 5.7, 3.0, 4.0 |
| M3.5 | standard (generated approximation) | 7.1 |
| M4 | standard, short | 8.1, 4.0 |
| M5 | standard, short | 9.5, 5.8 |
| M6 | standard | 12.7 |
| M8 | standard | 12.7 |

The plugin rejects unsupported combinations and does not create a mounting hole. Consult the [CNC Kitchen dimensions and design guidelines](https://cnckitchen.com/s/CNCKitchen_Heat-Set-Insert-Dimensions-and-Design-Guidelines.pdf) and calibrate a test print for the actual inserts and printer. SolidCraft may report tolerance or repair warnings for the supplied STEP geometry.

## Build and verify

Install Rust with `wasm32-unknown-unknown`, `wasm-tools`, `zip`, and `solidcraft-cli`, then run the following from this plugin's directory:

```sh
cargo test --lib
bash build.sh
solidcraft-cli plugin check dist/heatset-insert.zip
cp -r heatset-insert/tests dist/heatset-insert/
solidcraft-cli plugin test dist/heatset-insert --require-cases
python3 scripts/test_library.py "$(command -v solidcraft-cli)"
python3 scripts/test_joint.py "$(command -v solidcraft-cli)"
```
