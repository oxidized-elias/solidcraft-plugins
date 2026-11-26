# SolidCraft Plugins

This repository contains the source and build scripts for SolidCraft plugins:

- [Heat-Set Insert Generator](plugins/heatset-insert/README.md)
- [Aluminum Extrusions](plugins/aluminum-extrusions/README.md)
- [Linear Rails](plugins/linear-rails/README.md)
- [Breakaway Supports](plugins/breakaway-supports/README.md)

## Install a release

Download the ZIP for the plugin you want from [GitHub Releases](https://github.com/oxidized-elias/solidcraft-plugins/releases). Each plugin has its own ZIP; download that file directly and choose **Plugins → Install plugin…** in SolidCraft. The source-code archives on the release page are not plugin packages.

## Build and install

You need Python 3, Rust and Cargo, `wasm-tools`, and `zip` to build the plugins. Aluminum Extrusions, Linear Rails, and Breakaway Supports also need `solidcraft-cli` on `PATH` or in its standard per-user Linux Flatpak location. Install the Rust `wasm32-unknown-unknown` target with `rustup target add wasm32-unknown-unknown`.

Clone the repository and run the interactive installer in one command:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh
```

To build a plugin without the interactive menu, pass its name:

```sh
git clone https://github.com/oxidized-elias/solidcraft-plugins.git && ./solidcraft-plugins/install.sh heatset-insert
```

Use `aluminum-extrusions`, `linear-rails`, or `breakaway-supports` to choose another plugin. The clone commands create a `solidcraft-plugins` directory, so that directory must not already exist. From an existing checkout, run:

```sh
./install.sh heatset-insert
./install.sh aluminum-extrusions
./install.sh linear-rails
./install.sh breakaway-supports
```

The installer prints the ZIP location under `plugins/<plugin>/dist/`. In SolidCraft, choose **Plugins → Install plugin…** and select the ZIP. Run `./install.sh` without arguments to use the interactive menu. The installer builds from source and does not install prerequisites.

Each directory under `plugins/` contains a plugin’s source, tests, and build script. To add a plugin, create a directory there with a `build.sh` that produces an installable package. The shared installer discovers it automatically.

## Releases

A push to `main` or a manual run from the Actions tab builds and tests the plugins, then creates a release named for that commit. Push a version tag such as `v0.1.0` to create a versioned release instead. Each release contains the individual installable ZIPs as separate assets.

The workflow uses SolidCraft CLI 0.45.0 for its host checks. Update the workflow if the plugins require a newer host, and review the redistribution terms below before publishing.

## Redistribution

The plugins include third-party assets with separate attribution and unconfirmed redistribution terms. Read each plugin's README and obtain any necessary permissions before redistributing it. No license has been selected for the project code.
