# SolidCraft Plugins

Source and build scripts for SolidCraft plugins:

- [Heat-Set Insert Generator](plugins/heatset-insert/README.md)
- [Aluminum Extrusions](plugins/aluminum-extrusions/README.md)
- [Linear Rails](plugins/linear-rails/README.md)

## Build and install

Install Python 3, Rust and Cargo, the Rust `wasm32-unknown-unknown` target, `wasm-tools`, and `zip`. Aluminum Extrusions and Linear Rails also require `solidcraft-cli` on `PATH`, or its standard per-user Linux Flatpak installation. Install the Rust target with `rustup target add wasm32-unknown-unknown`.

Clone the repository and run the interactive installer in one command:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh
```

For unattended use, pass the plugin name on the same command:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh heatset-insert
```

Use `aluminum-extrusions` or `linear-rails` to select another plugin. These commands create a `solidcraft-plugins` directory in the current directory and require it not to exist already. From an existing checkout, run:

```sh
./install.sh heatset-insert
./install.sh aluminum-extrusions
./install.sh linear-rails
```

The command prints the ZIP location under `plugins/<plugin>/dist/`. In SolidCraft, choose **Plugins → Install plugin…** and select that ZIP. Run `./install.sh` without arguments for the interactive menu. The installer builds from source; it does not install software prerequisites.

Each plugin has its own source, tests, and build script under `plugins/`. To add a plugin, give it a directory there with a `build.sh` that creates an installable package; the shared installer discovers it automatically.

## Redistribution

The plugins include third-party assets with separate attribution and uncertain redistribution terms. Review each plugin's README and obtain any necessary permissions before redistributing them. No license has been selected for the project code.
