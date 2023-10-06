Neo4j image for [testcontainers](https://crates.io/crates/testcontainers).

This image is based on the official [Neo4j](https://hub.docker.com/_/neo4j) image.
The default user is `neo4j` and the default password is `neo`.
The default version is `5`.

# Example

```rust,no_run
use testcontainers::clients::Cli;
use neo4j_testcontainers::Neo4j;

let docker = Cli::default();
let container = docker.run(Neo4j::default());
let uri = container.image().bolt_uri_ipv4();
let auth_user = container.image().user();
let auth_pass = container.image().password();
// connect to Neo4j with the uri, user and pass
```

# Neo4j Version

The version of the image can be set with the `NEO4J_VERSION_TAG` environment variable.
The default version is `5`.
The available versions can be found on [Docker Hub](https://hub.docker.com/_/neo4j/tags).

The used version can be retrieved with the `version` method.

# Auth

The default user is `neo4j` and the default password is `neo`.

The used user can be retrieved with the `user` method.
The used password can be retrieved with the `pass` method.

# Environment variables

The following environment variables are supported:
  * `NEO4J_VERSION_TAG`: The default version of the image to use.
  * `NEO4J_TEST_USER`: The default user to use for authentication.
  * `NEO4J_TEST_PASS`: The default password to use for authentication.

# Neo4j Labs Plugins

Neo4j offers built-in support for Neo4j Labs plugins.
The method `with_neo4j_labs_plugin` can be used to define them.

Supported plugins are APOC, APOC Core, Bloom, Streams, Graph Data Science, and Neo Semantics.

# MSRV

The crate has a minimum supported Rust version (MSRV) of `1.63.0`.

A change in the MSRV in *not* considered a breaking change.
For versions past 1.0.0, a change in the MSRV can be done in a minor version increment (1.1.3 -> 1.2.0)
for versions before 1.0.0, a change in the MSRV can be done in a patch version increment (0.1.3 -> 0.1.4).
