# SolidCraft Plugins

This repository contains the source and build scripts for three SolidCraft plugins:

- [Heat-Set Insert Generator](plugins/heatset-insert/README.md)
- [Aluminum Extrusions](plugins/aluminum-extrusions/README.md)
- [Linear Rails](plugins/linear-rails/README.md)

## Build and install

You need Python 3, Rust and Cargo, `wasm-tools`, and `zip` to build the plugins. Aluminum Extrusions and Linear Rails also need `solidcraft-cli` on `PATH` or in its standard per-user Linux Flatpak location. Install the Rust `wasm32-unknown-unknown` target with `rustup target add wasm32-unknown-unknown`.

Clone the repository and run the interactive installer in one command:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh
```

To build a plugin without the interactive menu, pass its name:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh heatset-insert
```

Use `aluminum-extrusions` or `linear-rails` to choose another plugin. The clone commands create a `solidcraft-plugins` directory, so that directory must not already exist. From an existing checkout, run:

```sh
./install.sh heatset-insert
./install.sh aluminum-extrusions
./install.sh linear-rails
```

The installer prints the ZIP location under `plugins/<plugin>/dist/`. In SolidCraft, choose **Plugins → Install plugin…** and select the ZIP. Run `./install.sh` without arguments to use the interactive menu. The installer builds from source and does not install prerequisites.

Each directory under `plugins/` contains a plugin’s source, tests, and build script. To add a plugin, create a directory there with a `build.sh` that produces an installable package. The shared installer discovers it automatically.

## Releases

Push to `main` or run the workflow manually from the Actions tab to build and test the plugins. The resulting ZIPs are available as a workflow artifact. Push a version tag such as `v0.1.0` to also create a GitHub release with the ZIPs attached.

The workflow uses SolidCraft CLI 0.45.0 for its host checks. Update the workflow if the plugins require a newer host, and review the redistribution terms below before publishing.

## Redistribution

The plugins include third-party assets with separate attribution and unconfirmed redistribution terms. Read each plugin's README and obtain any necessary permissions before redistributing it. No license has been selected for the project code.
