#![doc = include_str!("../doc/lib.md")]
#![warn(clippy::all, clippy::nursery)]
#![allow(clippy::cargo, clippy::pedantic)]
#![allow(clippy::missing_const_for_fn)]
#![warn(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_copy_implementations,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2021_incompatible_or_patterns,
    rust_2021_incompatible_closure_captures,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unsafe_op_in_unsafe_fn,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused,
    while_true
)]

pub use testcontainers_modules::neo4j::{Neo4j, Neo4jImage, Neo4jLabsPlugin};

use std::{borrow::Cow, env::var};
use testcontainers::Container;

pub trait Neo4jExt: Sized {
    /// Create a new instance of a Neo4j 5 image with the default user and password.
    fn from_env() -> Neo4j;

    /// Create a new instance of a Neo4j image of the given version with the default user and password.
    ///
    /// # Panics
    ///
    /// If the version is not valid according to the format described in [`Self::with_version()`].
    #[deprecated(since = "0.2.0", note = "Use `from_env().with_version()` instead.")]
    #[must_use]
    fn from_version(version: &str) -> Neo4j {
        Self::from_env().with_version(version.to_owned())
    }

    /// Create a new instance of a Neo4j image with the version and given user and password.
    ///
    /// # Panics
    ///
    /// If the version is not valid according to the format described in [`Self::with_version()`].
    #[deprecated(
        since = "0.2.0",
        note = "Use `from_env().with_version().with_user().with_password()` instead."
    )]
    #[must_use]
    fn from_auth_and_version(version: &str, user: &str, pass: &str) -> Neo4j {
        Self::from_env()
            .with_version(version.to_owned())
            .with_user(user.to_owned())
            .with_password(pass.to_owned())
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv4.
    #[deprecated(
        since = "0.2.0",
        note = "Use `container.image().bolt_uri_ipv4()` instead."
    )]
    #[must_use]
    fn uri_ipv4(container: &Container<'_, Neo4jImage>) -> String {
        container.image().bolt_uri_ipv4()
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv6.
    #[deprecated(
        since = "0.2.0",
        note = "Use `container.image().bolt_uri_ipv6()` instead."
    )]
    #[must_use]
    fn uri_ipv6(container: &Container<'_, Neo4jImage>) -> String {
        container.image().bolt_uri_ipv6()
    }
}

impl Neo4jExt for Neo4j {
    fn from_env() -> Self {
        const DEFAULT_USER: &str = "neo4j";
        const DEFAULT_PASS: &str = "neo";
        const DEFAULT_VERSION_TAG: &str = "5";

        let user = var("NEO4J_TEST_USER").map_or_else(|_| Cow::Borrowed(DEFAULT_USER), Cow::Owned);
        let pass = var("NEO4J_TEST_PASS").map_or_else(|_| Cow::Borrowed(DEFAULT_PASS), Cow::Owned);
        let version = var("NEO4J_VERSION_TAG")
            .map_or_else(|_| Cow::Borrowed(DEFAULT_VERSION_TAG), Cow::Owned);

        Self::new()
            .with_user(user)
            .with_password(pass)
            .with_version(version)
    }
}

pub trait Neo4jImageExt {
    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv4.
    fn bolt_uri_ipv4(&self) -> String;

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv6.
    fn bolt_uri_ipv6(&self) -> String;

    /// Return the connection URI to connect to the Neo4j server via HTTP over IPv4.
    fn http_uri_ipv4(&self) -> String;

    /// Return the connection URI to connect to the Neo4j server via HTTP over IPv6.
    fn http_uri_ipv6(&self) -> String;
}

impl Neo4jImageExt for Neo4jImage {
    fn bolt_uri_ipv4(&self) -> String {
        format!("bolt://127.0.0.1:{}", self.bolt_port_ipv4())
    }

    fn bolt_uri_ipv6(&self) -> String {
        format!("bolt://[::1]:{}", self.bolt_port_ipv6())
    }

    fn http_uri_ipv4(&self) -> String {
        format!("http://127.0.0.1:{}", self.http_port_ipv4())
    }

    fn http_uri_ipv6(&self) -> String {
        format!("http://[::1]:{}", self.http_port_ipv6())
    }
}
