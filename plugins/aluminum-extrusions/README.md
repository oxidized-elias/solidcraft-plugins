# Aluminum Extrusions for SolidCraft

Create a SolidCraft extrusion at a length you choose in millimeters. The included profiles are 1515, 2020, 2020 C-channel, 2040, 4040 and 4040 Rounded, based on the supplied SolidCraft designs.

## Install and use

Download `aluminum-extrusions.zip` from [GitHub Releases](https://github.com/oxidized-elias/solidcraft-plugins/releases) and select it in SolidCraft under **Plugins → Install plugin…**. To build it yourself:

From the repository root, run `./install.sh aluminum-extrusions`. In SolidCraft, choose **Plugins → Install plugin…** and select `dist/aluminum-extrusions.zip` from this plugin’s directory. Open a design, choose **Extrusions** on the ribbon, select a profile and enter a positive length.

The plugin creates a sketch and an extrusion, along with any fillets in the source profile. Use Undo to remove the insertion, or edit the extrusion feature to change its length.

## Adding profiles

Copy a `.sc` file into `profiles/`, then run `python3 scripts/build.py` and reinstall the resulting ZIP. The file name becomes the profile name. Each source must contain one active XY sketch, one extrusion and optional fillets. The sketch must use closed contours made of lines, circles and arcs. Fillet radii must be numeric and their edge picks must lie on edges parallel to the extrusion. Test each new profile in SolidCraft before using it: the extractor interprets arcs as the shorter path and approximates them using straight segments of at most 3 degrees. The extractor rejects open contours and unsupported features. Rebuild the package after adding a profile because SolidCraft plugins cannot currently browse an arbitrary user profile folder without filesystem grants.

## Build from source

Install Python 3, Rust, `wasm-tools`, and `solidcraft-cli` from SolidCraft. Add the WebAssembly target, then build:

```sh
rustup target add wasm32-unknown-unknown
python3 scripts/build.py
```

The build regenerates the profile data and manifest, compiles the plugin, validates the component, and runs the SolidCraft host tests. It then creates `dist/aluminum-extrusions.zip`. SolidCraft CLI must be on `PATH`; the script also detects the per-user Linux Flatpak installation. Generated packages, test results and Rust build outputs are excluded from source control.

## Geometry notes

The supplied 4040 design has a broken fillet reference in the installed SolidCraft version; this plugin reapplies its four saved corner fillets by position. The rounded profile uses chords to approximate its arcs, so its geometry is not exact. Check dimensions and fit against the manufacturer's drawing before fabrication.

## Attribution and licensing

`wit/plugin.wit` is the SolidCraft plugin interface, copied from [SolidCraft](https://github.com/tangency-labs/solidcraft/blob/e8689fd86eac15513db4d91e8b764bf036884d6b/crates/craft-plugin/wit/plugin.wit). It is available under MIT or Apache-2.0; the [MIT notice](LICENSES/SolidCraft-MIT.txt) is included here. The original `.sc` extrusion profiles were created by **Hellspark** (Discord: `_hellspark`); thank you for providing the base designs. Their redistribution terms have not been established. No license has yet been selected for the rest of this project.
