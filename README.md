substrate-deps
==============

[![rust build](https://github.com/paritytech/substrate-deps/workflows/rust/badge.svg)](https://github.com/paritytech/substrate-deps/actions)
[![dependency status](https://deps.rs/repo/github/paritytech/substrate-deps/status.svg)](https://deps.rs/repo/github/paritytech/substrate-deps)

`substrate-deps` is a command line tool for managing [Parity Substrate](http://substrate.dev) pallet dependencies.
It allows adding a new pallet to your runtime, and applying a default configuration so you can start hacking right away.
It uses [metadata](#Substrate-Runtime-module-metadata-model) defined in the Cargo.toml manifest of Susbstrate runtime modules.

The following commands are available / planned:

- [`substrate-deps add`](#substrate-deps-add)
- [`substrate-deps graph`](#substrate-deps-graph)

## How to install

Install `substrate-deps` locally with:
```bash
cargo install substrate-deps
```

## Commands

### `substrate-deps add`

Add a new pallet dependency to your Substrate runtime's `Cargo.toml`.

#### Examples

To add an hypothetical `template-module` that depends on the `pallet-balances` pallet:
```sh
$ # Add the pallet template-module to the runtime whose manifest is specified as argument, using the specified alternative registry.
$ substrate-deps add template-module --manifest-path ../substrate-package/substrate-node-template/runtime/Cargo.toml

No metadata found for pallet pallet-balances
Added pallet pallet-balances v2.0.0 configuration in your node runtime.
Added pallet template-module v0.2.1 as dependency in your node runtime manifest.
Added pallet template-module v0.2.1 configuration in your node runtime.
```

#### Usage

```plain
$ substrate-deps add --help
USAGE:
    substrate-deps add [FLAGS] [OPTIONS] <pallet>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      No output printed to stdout
    -v, --verbose    Use verbose output
    -V, --version    Prints version information

OPTIONS:
        --manifest-path <path>    Path to the manifest of the runtime. [default: Cargo.toml]
        --registry <registry>     Registry to use. [default: crates-io]

ARGS:
    <pallet>    Pallet to be added e.g. pallet-staking
```

This command allows you to add a new pallet dependency to your Substrate runtime's Cargo.toml manifest file. `substrate-deps add` will fetch the pallet, parse its metadata if any, and add it plus any related depencies, as well as apply default module & trait configuration to your runtime's `libs.rs` file.

### `substrate-deps graph`

Generates a dependency graph of the pallets used by your Substrate runtime.

#### Examples

This command output a dependency graph for [graphviz](https://graphviz.gitlab.io/download/), please make sure your have it install to be able to generate an image file with the instruction below.

```sh
$ # Generate a dependency graph of the pallets used by the runtime whose manifest is specified as argument and pipe it to the dot command to generate an image file.
$ substrate-deps graph --manifest-path ../substrate-package/substrate-node-template/runtime/Cargo.toml | dot -Tpng > graph.png
```

#### Usage
```plain
$ substrate-deps graph --help
substrate-deps-graph
Generate a graph of the Substrate runtime pallet dependencies.

USAGE:
    substrate-deps graph [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                Prints help information
    -I, --include-versions    Include the dependency version on nodes
    -q, --quiet               No output printed to stdout
    -v, --verbose             Use verbose output
    -V, --version             Prints version information

OPTIONS:
    --manifest-path <path>    Path to the manifest of the runtime. [default: Cargo.toml]
```

### Substrate Runtime pallet metadata model

`substrate-deps` uses metadata defined in module's Cargo.toml manifest to know about module trait dependencies, and to be be able to generate a default configuration for the module's configuration trait.

The metadata are defined in `[package.metadata.substrate]` table as follows:
```toml
[package.metadata.substrate]
# indicates which version of Substrate the module is compatible with
substrate_version = '2.0'
# Alias name of the module e.g. balances instead of paint-balances
module_alias = 'template'
# label describing the module purpose (for future use in a GUI)
module_label = 'Template Module for Substrate'
# icon representing the module (for future use in a GUI)
icon = 'gear.png'
# category the module should be classified into (for future use in a GUI)
# default categories: accounts, assets, consensus, governance, runtime, smart contracts, example
module_categories = ['example']
# Defines a list of dependent modules used when applying a default configuration for the current module. The modules referenced here will be added as dependencies in the runtime's manifest (in addition to the request module).
module_deps_defaults = ['Balances:paint-balances']
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
