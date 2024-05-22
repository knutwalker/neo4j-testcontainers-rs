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

#[cfg(not(test))]
use std::io::BufRead;
use std::{borrow::Cow, env::var};
use testcontainers_modules::testcontainers::Container;
pub use testcontainers_modules::{
    neo4j::{Neo4j, Neo4jImage},
    testcontainers::clients,
    testcontainers::RunnableImage,
};

/// The prelude only exports the extension traits, so that you can selectively import from the root
/// module while still being able to wildcard-use all the extension traits.
pub mod prelude {
    pub use super::{Neo4jExt, Neo4jImageExt, Neo4jRunnableImageExt};
}

/// Extension trait for the [`Neo4j`] type, adding a [`Neo4jExt::from_env`] constructor.
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

/// Extension trait for the [`Neo4jImage`] type, adding convenience methods to access to Bolt or
/// HTTP ports.
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

/// Extension trait for [`RunnableImage<Neo4jImage>`] to allow the usage of Neo4j Enterprise
/// Edition via [`Neo4jRunnableImageExt::with_enterprise_edition`].
pub trait Neo4jRunnableImageExt: Sized {
    /// Use the enterprise edition of Neo4j.
    ///
    /// # Note
    /// Please have a look at the [Neo4j Licensing page](https://neo4j.com/licensing/).
    /// While the Neo4j Community Edition can be used for free in your projects under the GPL v3 license,
    /// Neo4j Enterprise edition needs either a commercial, education or evaluation license.
    fn with_enterprise_edition(
        self,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send + 'static>>;
}

impl Neo4jRunnableImageExt for RunnableImage<Neo4jImage> {
    fn with_enterprise_edition(
        self,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send + 'static>> {
        const ACCEPTANCE_FILE_NAME: &str = "container-license-acceptance.txt";

        let version = self.descriptor();

        if version.ends_with("-enterprise")
            || self
                .env_vars()
                .any(|(k, v)| k == "NEO4J_ACCEPT_LICENSE_AGREEMENT" && v == "yes")
        {
            return Ok(self);
        }

        let (name, version) = version.split_once(':').unwrap();

        let version = format!("{}-enterprise", version);
        let image = format!("{}:{}", name, version);

        let acceptance_file = std::env::current_dir()
            .ok()
            .map(|o| o.join(ACCEPTANCE_FILE_NAME));

        #[cfg(test)]
        let has_license_acceptance = true;

        #[cfg(not(test))]
        let has_license_acceptance = acceptance_file
            .as_deref()
            .and_then(|o| std::fs::File::open(o).ok())
            .into_iter()
            .flat_map(|o| std::io::BufReader::new(o).lines())
            .any(|o| o.map_or(false, |line| line.trim() == image.as_str()));

        if !has_license_acceptance {
            return Err(format!(
                concat!(
                    "You need to accept the Neo4j Enterprise Edition license by ",
                    "creating the file `{}` with the following content:\n\n\t{}",
                ),
                acceptance_file.map_or_else(
                    || ACCEPTANCE_FILE_NAME.to_owned(),
                    |o| { o.display().to_string() }
                ),
                image
            )
            .into());
        }

        let this = self
            .with_env_var(("NEO4J_ACCEPT_LICENSE_AGREEMENT", "yes"))
            .with_tag(version);
        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_enterprise_version() {
        let img = RunnableImage::from(Neo4j::default());
        let img = img.with_enterprise_edition().unwrap();

        let version = img.descriptor();
        assert_eq!(version, "neo4j:5-enterprise");

        let env_var = img.env_vars().find_map(|(k, v)| {
            if k == "NEO4J_ACCEPT_LICENSE_AGREEMENT" {
                Some(v.clone())
            } else {
                None
            }
        });
        assert_eq!(env_var.as_deref(), Some("yes"))
    }
}
