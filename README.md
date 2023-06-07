# neo4j_testcontainers [![CI Status][ci-badge]][ci-url] [![Crates.io][crates-badge]][crates-url] [![Docs][docs-badge]][docs-url] ![License: MIT][license-badge] ![Rust Version: ^1.60.0][rust-version-badge]

[ci-badge]: https://github.com/knutwalker/neo4j-testcontainers-rs/actions/workflows/checks.yml/badge.svg
[ci-url]: https://github.com/knutwalker/neo4j-testcontainers-rs
[crates-badge]: https://img.shields.io/crates/v/neo4j_testcontainers?style=shield
[crates-url]: https://crates.io/crates/neo4j_testcontainers
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/neo4j_testcontainers
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg?style=shield
[rust-version-badge]: https://img.shields.io/badge/rustc-%5E1.60.0-orange.svg?style=shield

Neo4j image for [testcontainers][__link0].

This image is based on the official [Neo4j][__link1] image. The default user is `neo4j` and the default password is `neo`. The default version is `5`.


## Example


```rust
use testcontainers::clients::Cli;
use neo4j_testcontainers::Neo4j;

let cli = Cli::default();
let container = docker.run(Neo4j::default());
let uri = Neo4j::uri_ipv4(&container);
let auth_user = container.image().user();
let auth_pass = container.image().pass();
// connect to Neo4j with the uri, user and pass
```


## Neo4j Version

The version of the image can be set with the `NEO4J_VERSION_TAG` environment variable. The default version is `5`. The available versions can be found on [Docker Hub][__link2].

The used version can be retrieved with the `version` method.


## Auth

The default user is `neo4j` and the default password is `neo`.

The used user can be retrieved with the `user` method. The used password can be retrieved with the `pass` method.


## Environment variables

The following environment variables are supported:

 - `NEO4J_VERSION_TAG`: The default version of the image to use.
 - `NEO4J_TEST_USER`: The default user to use for authentication.
 - `NEO4J_TEST_PASS`: The default password to use for authentication.


## MSRV

The crate has a minimum supported Rust version (MSRV) of `1.60.0`.

A change in the MSRV in *not* considered a breaking change. For versions past 1.0.0, a change in the MSRV can be done in a minor version increment (1.1.3 -> 1.2.0) for versions before 1.0.0, a change in the MSRV can be done in a patch version increment (0.1.3 -> 0.1.4).



## License

neo4j_testcontainers is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

---
 [__link0]: https://crates.io/crates/testcontainers
 [__link1]: https://hub.docker.com/_/neo4j
 [__link2]: https://hub.docker.com/_/neo4j/tags
