substrate-deps
==============

[![rust build](https://github.com/stiiifff/substrate-deps/workflows/rust/badge.svg)](https://github.com/stiiifff/substrate-deps/actions)

`substrate-deps` is a (experimental) command line tool for managing [Parity Substrate](http://substrate.dev) runtime module dependencies.
It allows adding a new module to your runtime, and applying a default configuration so you can start hacking right away.
It uses [metadata](#Substrate-Runtime-module-metadata-model) defined in the Cargo.toml manifest of Susbstrate runtime modules.

The following commands are available / planned:

- [`substrate-deps add`](#substrate-deps-add)
- [`substrate-deps graph`](#substrate-deps-graph)

**Disclaimer**: This is a work in progress ! There are currently a few crucial pieces that are missing / unfinished for this tool to work properly:
- [ ] Semantic versioning for Substrate SRML modules
- [ ] Substrate SRML modules published on crates.io, or an alternative registry (*)
- [ ] cargo loads dependencies from crates.io instead of alt. registry when fetching a crate from an alt. registry

(*) For now, an alternative registry with Substrate SRML modules is available at https://dl.cloudsmith.io/public/steve-degosserie/substrate-mods/cargo/index.git

## How to install

```sh
cargo install substrate-deps
```

## Commands

### `substrate-deps add`

Add a new module dependency to your Substrate runtime's `Cargo.toml`.

#### Examples

To add an hypothetical `scml-template-module` that depends on the `srml-balances`module:
```sh
$ # Add the module scml-template-module to the runtime whose manifest is specified as argument, using the specified alternative registry.
$ substrate-deps add scml-template-module --manifest-path ../substrate-package/substrate-node-template/runtime/Cargo.toml --registry substrate-mods

Using registry 'substrate-mods' at: https://dl.cloudsmith.io/public/steve-degosserie/substrate-mods/cargo/index.git
    Updating 'https://dl.cloudsmith.io/public/steve-degosserie/substrate-mods/cargo/index.git' index
No metadata found for module srml-balances
Added module srml-balances v2.0.0 configuration in your node runtime.
Added module scml-template-module v0.2.1 as dependency in your node runtime manifest.
Added module scml-template-module v0.2.1 configuration in your node runtime.
```

#### Usage

```plain
$ substrate-deps add --help
USAGE:
    substrate-deps add [FLAGS] [OPTIONS] <module>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      No output printed to stdout
    -v, --verbose    Use verbose output
    -V, --version    Prints version information

OPTIONS:
        --manifest-path <path>    Path to the manifest of the runtime. [default: Cargo.toml]
        --registry <registry>     Registry to use [default: substrate-mods]

ARGS:
    <module>    Module to be added e.g. srml-staking
```

This command allows you to add a new module dependency to your Substrate runtime's Cargo.toml manifest file. For now, `<module>` should be the name of a module hosted on an alternative metionned in the `<registry>` argument. `substrate-deps add` will fetch the dmoeul, parse its metadata if any, and add it plus any related depencies, as well as apply default module & trait configuration to your runtime's `libs.rs` file.

### `substrate-deps graph`

Generates a dependency graph of the modules used by your Substrate runtime.

#### Examples

This command output a dependency graph for [graphviz](https://graphviz.gitlab.io/download/), please make sure your have it install to be able to generate an image file with the instruction below.

```sh
$ # Generate a dependency graph of the modules used by the runtime whose manifest is specified as argument and pipe it to the dot command to generate an image file.
$ substrate-deps graph --manifest-path ../substrate-package/substrate-node-template/runtime/Cargo.toml | dot -Tpng > graph.png
```

### Substrate Runtime module metadata model

`substrate-deps` uses metadata defined in module's Cargo.toml manifest to know about module trait dependencies, and to be be able to generate a default configuration for the module's configuration trait.

The metadata are defined in `[package.metadata.substrate]` table as follows:
```toml
[package.metadata.substrate]
# indicates which version of Substrate the module is compatible with
substrate_version = '2.0'
# Alias name of the module e.g. balances instead of srml-balances
module_alias = 'template'
# label describing the module purpose (for future use in a GUI)
module_label = 'Template Module for Substrate'
# icon representing the module (for future use in a GUI)
icon = 'gear.png'
# category the module should be classified into (for future use in a GUI)
# default categories: accounts, assets, consensus, governance, runtime, smart contracts, example
module_categories = ['example']
# Defines a list of dependent modules used when applying a default configuration for the current module. The modules referenced here will be added as dependencies in the runtime's manifest (in addition to the request module).
module_deps_defaults = ['Balances:srml-balances']
# Define a list of 'trait dependencies', that is, those traits being used when applying a default configuration for the module's configuration trait in the runtime lib.rs file.
trait_deps_defaults = ['Currency=Balances','Event=Event']
# Define the list of types exposed by the module, when configured in the construct_runtime! macro in the the runtime's lib.rs file.
module_cfg_defaults = ['Module','Call','Storage','Event<T>']
```

### License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
