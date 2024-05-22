# neo4j_testcontainers [![CI Status][ci-badge]][ci-url] [![Crates.io][crates-badge]][crates-url] [![Docs][docs-badge]][docs-url] ![License: MIT][license-badge] ![Rust Version: 1.63.0][rust-version-badge]

[ci-badge]: https://github.com/knutwalker/neo4j-testcontainers-rs/actions/workflows/checks.yml/badge.svg
[ci-url]: https://github.com/knutwalker/neo4j-testcontainers-rs
[crates-badge]: https://img.shields.io/crates/v/neo4j_testcontainers?style=shield
[crates-url]: https://crates.io/crates/neo4j_testcontainers
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/neo4j_testcontainers
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg?style=shield
[rust-version-badge]: https://img.shields.io/badge/rustc-1.63.0-orange.svg?style=shield

Neo4j image for [testcontainers][__link0].

This image is based on the official [Neo4j][__link1] image.
The default user is `neo4j` and the default password is `neo`.
The default version is `5`.

## Example

```rust
use neo4j_testcontainers::{clients::Cli, prelude::*, Neo4j};

let client = Cli::default();
let container = client.run(Neo4j::default());
let uri = container.image().bolt_uri_ipv4();
let auth_user = container.image().user();
let auth_pass = container.image().password();
// connect to Neo4j with the uri, user and pass
```

## Testcontainers Module

This crate builds on top of the `neo4j` image from [testcontainers-module][__link2].
It provides a few extension methods that are too specific to Neo4j to be included in the generic testcontainers module.

This crate also exports a few types that are required to get you started without having to add extra dependencies to either `testcontainers-modules` or `testcontainers`:

```rust
// Those types are exported
pub use testcontainers_modules::{
    neo4j::{Neo4j, Neo4jImage},
    testcontainers::clients,
    testcontainers::RunnableImage,
};
```

```rust
// ... on your end: use the exported types
use neo4j_testcontainers::{clients::Cli, Neo4j, RunnableImage};
```

## Neo4j Version

The version of the image can be set with the `NEO4J_VERSION_TAG` environment variable.
The default version is `5`.
The available versions can be found on [Docker Hub][__link3].

The used version can be retrieved with the `version` method.

## Auth

The default user is `neo4j` and the default password is `neo`.

The used user can be retrieved with the `user` method.
The used password can be retrieved with the `pass` method.

## Environment variables

The following environment variables are supported:

* `NEO4J_VERSION_TAG`: The default version of the image to use.
* `NEO4J_TEST_USER`: The default user to use for authentication.
* `NEO4J_TEST_PASS`: The default password to use for authentication.

## Neo4j Labs Plugins

Neo4j offers built-in support for Neo4j Labs plugins.
The method `with_neo4j_labs_plugin` can be used to define them.

Supported plugins are APOC, APOC Core, Bloom, Streams, Graph Data Science, and Neo Semantics.

## Enterprise edition

In order to use Neo4j Enterprise Edition for the `testcontainer`, you can configure it on the `RunnableImage`:

```rust
use neo4j_testcontainers::{clients::Cli, prelude::*, Neo4j, RunnableImage};

let client = Cli::default();
let neo4j = RunnableImage::from(Neo4j::default());
let neo4j = neo4j.with_enterprise_edition().expect("license not accepted");
let container = client.run(neo4j);
```

Before enabling this, have a read through the [Neo4j Licensing page][__link4] to understand the terms
under which you can use the Enterprise edition.

You can accept the terms and condition of the enterprise version by adding a file named `container-license-acceptance.txt` to the root of your cargo workspace, containing the text `neo4j:5-enterprise` in one line.
The content of the file must be the same as the actual image that is being used, so if you change the version, you also need to change to content of this file.

## MSRV

The crate has a minimum supported Rust version (MSRV) of `1.63.0`.

A change in the MSRV in *not* considered a breaking change.
For versions past 1.0.0, a change in the MSRV can be done in a minor version increment (1.1.3 -> 1.2.0)
for versions before 1.0.0, a change in the MSRV can be done in a patch version increment (0.1.3 -> 0.1.4).


## License

neo4j_testcontainers is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

---
 [__link0]: https://crates.io/crates/testcontainers
 [__link1]: https://hub.docker.com/_/neo4j
 [__link2]: https://crates.io/crates/testcontainers-modules
 [__link3]: https://hub.docker.com/_/neo4j/tags
 [__link4]: https://neo4j.com/licensing/
