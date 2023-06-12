#![doc = include_str!("../doc/lib.md")]
#![warn(clippy::all, clippy::nursery)]
#![allow(clippy::cargo, clippy::pedantic)]
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
#![allow(
    clippy::module_name_repetitions,
    clippy::bool_assert_comparison,
    clippy::missing_const_for_fn
)]

use std::{borrow::Cow, cell::RefCell, collections::HashMap};
use testcontainers::{
    core::{ContainerState, WaitFor},
    Container, Image, RunnableImage,
};

/// Available Neo4j plugins.
/// See [Neo4j operations manual](https://neo4j.com/docs/operations-manual/current/docker/operations/#docker-neo4j-plugins) for more information.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Neo4jLabsPlugin {
    Apoc,
    ApocCore,
    Bloom,
    Streams,
    GraphDataScience,
    NeoSemantics,
    Custom(String),
}

impl std::fmt::Display for Neo4jLabsPlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apoc => formatter.pad("apoc"),
            Self::ApocCore => formatter.pad("apoc-core"),
            Self::Bloom => formatter.pad("bloom"),
            Self::Streams => formatter.pad("streams"),
            Self::GraphDataScience => formatter.pad("graph-data-science"),
            Self::NeoSemantics => formatter.pad("n10s"),
            Self::Custom(plugin_name) => formatter.pad(plugin_name),
        }
    }
}

#[doc = include_str!("../doc/lib.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neo4j {
    version: Value,
    user: Value,
    pass: Value,
    plugins: Vec<Neo4jLabsPlugin>,
}

impl Neo4j {
    const DEFAULT_USER: &str = "neo4j";
    const DEFAULT_PASS: &str = "neo";
    const DEFAULT_VERSION_TAG: &str = "5";

    /// Create a new instance of a Neo4j image.
    pub fn new() -> Self {
        Self {
            version: Value::Default(Self::DEFAULT_VERSION_TAG),
            user: Value::Default(Self::DEFAULT_USER),
            pass: Value::Default(Self::DEFAULT_PASS),
            plugins: Vec::new(),
        }
    }

    /// Create a new instance of a Neo4j 5 image with the default user and password.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            version: Value::Env {
                var: "NEO4J_VERSION_TAG",
                fallback: Self::DEFAULT_VERSION_TAG,
            },
            user: Value::Env {
                var: "NEO4J_TEST_USER",
                fallback: Self::DEFAULT_USER,
            },
            pass: Value::Env {
                var: "NEO4J_TEST_PASS",
                fallback: Self::DEFAULT_PASS,
            },
            plugins: Vec::new(),
        }
    }

    /// Set the Neo4j version to use.
    /// The value must be an existing Neo4j version tag.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let version: String = version.into();
        self.version = Value::Value(version);
        self
    }

    /// Set the username to use.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Value::Value(user.into());
        self
    }

    /// Set the password to use.
    pub fn with_password(mut self, pass: impl Into<String>) -> Self {
        self.pass = Value::Value(pass.into());
        self
    }

    /// Add Neo4j lab plugins to get started with the database.
    pub fn with_neo4j_labs_plugin(mut self, plugins: &[Neo4jLabsPlugin]) -> Self {
        self.plugins.extend_from_slice(plugins);
        self
    }

    /// Create a new instance of a Neo4j image of the given version with the default user and password.
    #[deprecated(since = "0.2.0", note = "Use `from_env().with_version()` instead.")]
    #[must_use]
    pub fn from_version(version: &str) -> Self {
        Self::from_env().with_version(version)
    }

    /// Create a new instance of a Neo4j image with the version and given user and password.
    #[deprecated(
        since = "0.2.0",
        note = "Use `from_env().with_version().with_user().with_password()` instead."
    )]
    #[must_use]
    pub fn from_auth_and_version(version: &str, user: &str, pass: &str) -> Self {
        Self::from_env()
            .with_version(version)
            .with_user(user)
            .with_password(pass)
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv4.
    #[deprecated(
        since = "0.2.0",
        note = "Use `container.image().bolt_uri_ipv4()` instead."
    )]
    #[must_use]
    pub fn uri_ipv4(container: &Container<'_, Neo4jImage>) -> String {
        container.image().bolt_uri_ipv4()
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv6.
    #[deprecated(
        since = "0.2.0",
        note = "Use `container.image().bolt_uri_ipv6()` instead."
    )]
    #[must_use]
    pub fn uri_ipv6(container: &Container<'_, Neo4jImage>) -> String {
        container.image().bolt_uri_ipv6()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Env {
        var: &'static str,
        fallback: &'static str,
    },
    Default(&'static str),
    Value(String),
}

impl Default for Neo4j {
    fn default() -> Self {
        Self::from_env()
    }
}

/// The actual Neo4j testcontainers image type which is returned by `container.image()`
pub struct Neo4jImage {
    version: String,
    user: String,
    pass: String,
    env_vars: HashMap<String, String>,
    state: RefCell<Option<ContainerState>>,
}

impl Neo4jImage {
    /// Return the version of the Neo4j image.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Return the user of the Neo4j server.
    #[must_use]
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Return the password of the Neo4j server.
    #[must_use]
    pub fn pass(&self) -> &str {
        &self.pass
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv4.
    #[must_use]
    pub fn bolt_uri_ipv4(&self) -> String {
        let bolt_port = self
            .state
            .borrow()
            .as_ref()
            .expect("Container must be started before URI can be retrieved")
            .host_port_ipv4(7687);
        format!("bolt://127.0.0.1:{}", bolt_port)
    }

    /// Return the connection URI to connect to the Neo4j server via Bolt over IPv6.
    #[must_use]
    pub fn bolt_uri_ipv6(&self) -> String {
        let bolt_port = self
            .state
            .borrow()
            .as_ref()
            .expect("Container must be started before URI can be retrieved")
            .host_port_ipv6(7687);
        format!("bolt://[::1]:{}", bolt_port)
    }

    /// Return the connection URI to connect to the Neo4j server via HTTP over IPv4.
    #[must_use]
    pub fn http_uri_ipv4(&self) -> String {
        let http_port = self
            .state
            .borrow()
            .as_ref()
            .expect("Container must be started before URI can be retrieved")
            .host_port_ipv4(7474);
        format!("http://127.0.0.1:{}", http_port)
    }

    /// Return the connection URI to connect to the Neo4j server via HTTP over IPv6.
    #[must_use]
    pub fn http_uri_ipv6(&self) -> String {
        let http_port = self
            .state
            .borrow()
            .as_ref()
            .expect("Container must be started before URI can be retrieved")
            .host_port_ipv6(7474);
        format!("http://[::1]:{}", http_port)
    }
}

impl Image for Neo4jImage {
    type Args = ();

    fn name(&self) -> String {
        "neo4j".to_owned()
    }

    fn tag(&self) -> String {
        self.version.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Bolt enabled on"),
            WaitFor::message_on_stdout("Started."),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn exec_after_start(&self, cs: ContainerState) -> Vec<testcontainers::core::ExecCommand> {
        *self.state.borrow_mut() = Some(cs);
        Vec::new()
    }
}

impl Neo4j {
    fn auth_env(&self) -> impl IntoIterator<Item = (String, String)> {
        let user = Self::value(&self.user);
        let pass = Self::value(&self.pass);

        Some(("NEO4J_AUTH".to_owned(), format!("{}/{}", user, pass)))
    }

    fn plugins_env(&self) -> impl IntoIterator<Item = (String, String)> {
        if self.plugins.is_empty() {
            return None;
        }

        let plugin_names = self
            .plugins
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<String>>()
            .join(",");

        let plugin_definition = format!("[{}]", plugin_names);

        Some(("NEO4JLABS_PLUGINS".to_owned(), plugin_definition))
    }

    fn conf_env(&self) -> impl IntoIterator<Item = (String, String)> {
        let pass = Self::value(&self.pass);

        if pass.len() < 8 {
            Some((
                "NEO4J_dbms_security_auth__minimum__password__length".to_owned(),
                pass.len().to_string(),
            ))
        } else {
            None
        }
    }

    fn build(mut self) -> Neo4jImage {
        self.plugins.sort();
        self.plugins.dedup();

        let mut env_vars = HashMap::new();

        for (key, value) in self.auth_env() {
            env_vars.insert(key, value);
        }

        for (key, value) in self.plugins_env() {
            env_vars.insert(key, value);
        }

        for (key, value) in self.conf_env() {
            env_vars.insert(key, value);
        }

        Neo4jImage {
            version: Self::value(&self.version).into_owned(),
            user: Self::value(&self.user).into_owned(),
            pass: Self::value(&self.pass).into_owned(),
            env_vars,
            state: RefCell::new(None),
        }
    }

    fn value(value: &Value) -> Cow<'_, str> {
        match value {
            &Value::Env { var, fallback } => {
                std::env::var(var).map_or_else(|_| fallback.into(), Into::into)
            }
            &Value::Default(value) => value.into(),
            Value::Value(value) => value.as_str().into(),
        }
    }
}

impl From<Neo4j> for Neo4jImage {
    fn from(neo4j: Neo4j) -> Self {
        neo4j.build()
    }
}

impl From<Neo4j> for RunnableImage<Neo4jImage> {
    fn from(neo4j: Neo4j) -> Self {
        Self::from(neo4j.build())
    }
}

impl std::fmt::Debug for Neo4jImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neo4jImage")
            .field("version", &self.version)
            .field("user", &self.user)
            .field("pass", &self.pass)
            .field("env_vars", &self.env_vars)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_plugin_definition() {
        let neo4j = Neo4j::default()
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc])
            .build();
        assert_eq!(
            neo4j.env_vars.get("NEO4JLABS_PLUGINS").unwrap(),
            "[\"apoc\"]"
        );
    }

    #[test]
    fn multiple_plugin_definition() {
        let neo4j = Neo4j::default()
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc, Neo4jLabsPlugin::Bloom])
            .build();
        assert_eq!(
            neo4j.env_vars.get("NEO4JLABS_PLUGINS").unwrap(),
            "[\"apoc\",\"bloom\"]"
        );
    }

    #[test]
    fn multiple_wiht_plugin_calls() {
        let neo4j = Neo4j::default()
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc])
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Bloom])
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc])
            .build();
        assert_eq!(
            neo4j.env_vars.get("NEO4JLABS_PLUGINS").unwrap(),
            "[\"apoc\",\"bloom\"]"
        );
    }
}
