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

use std::{borrow::Cow, cell::RefCell, collections::HashMap, io::BufRead};
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
    enterprise: bool,
    plugins: Vec<Neo4jLabsPlugin>,
}

impl Neo4j {
    const DEFAULT_USER: &'static str = "neo4j";
    const DEFAULT_PASS: &'static str = "neo";
    const DEFAULT_VERSION_TAG: &'static str = "5";

    /// Create a new instance of a Neo4j image.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            version: Value::Default(Self::DEFAULT_VERSION_TAG),
            user: Value::Default(Self::DEFAULT_USER),
            pass: Value::Default(Self::DEFAULT_PASS),
            enterprise: false,
            plugins: Vec::new(),
        }
    }

    /// Create a new instance of a Neo4j 5 image with the default user and password.
    #[must_use]
    pub const fn from_env() -> Self {
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
            enterprise: false,
            plugins: Vec::new(),
        }
    }

    /// Set the Neo4j version to use.
    /// The value must be an existing Neo4j version tag.
    ///
    /// Only a subset of Semantic Versions are supported.
    /// The version must be of the format
    ///
    ///    MAJOR[.MINOR[.PATCH]]
    ///
    ///
    /// # Errors
    ///
    /// If the version is not valid according to the format.
    pub fn with_version(
        mut self,
        version: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let version: String = version.into();

        let version_valid =
            lenient_semver::parse_into::<'_, ValidateVersion>(&version).unwrap_or(false);
        if !version_valid {
            return Err(format!("Invalid version: {}", version).into());
        }

        self.version = Value::Value(version);
        Ok(self)
    }

    /// Set the username to use.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Value::Value(user.into());
        self
    }

    /// Set the password to use.
    #[must_use]
    pub fn with_password(mut self, pass: impl Into<String>) -> Self {
        self.pass = Value::Value(pass.into());
        self
    }

    /// Do not use any authentication on the testcontainer.
    ///
    /// Setting this will override any prior usages of [`Self::with_user`] and
    /// [`Self::with_password`].
    pub fn without_authentication(mut self) -> Self {
        self.user = Value::Unset;
        self.pass = Value::Unset;
        self
    }

    /// Use the enterprise edition of Neo4j.
    ///
    /// # Note
    /// Please have a look at the [Neo4j Licensing page](https://neo4j.com/licensing/).
    /// While the Neo4j Community Edition can be used for free in your projects under the GPL v3 license,
    /// Neo4j Enterprise edition needs either a commercial, education or evaluation license.
    pub fn with_enterprise_edition(
        mut self,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send + 'static>> {
        const ACCEPTANCE_FILE_NAME: &str = "container-license-acceptance.txt";

        let version = Self::value(&self.version).expect("Version is always set");
        let image = format!("neo4j:{}-enterprise", version);

        let has_license_acceptance = std::env::current_dir()
            .ok()
            .map(|o| o.join(ACCEPTANCE_FILE_NAME))
            .and_then(|o| std::fs::File::open(o).ok())
            .into_iter()
            .flat_map(|o| std::io::BufReader::new(o).lines())
            .any(|o| o.map_or(false, |line| line.trim() == image));

        if !has_license_acceptance {
            return Err(format!(
                concat!(
                    "You need to accept the Neo4j Enterprise Edition license ",
                    "by creating a file named `{}` in the current directory ",
                    "and adding the following line to it:\n\n\t{}",
                ),
                ACCEPTANCE_FILE_NAME, image
            )
            .into());
        }

        self.enterprise = true;
        Ok(self)
    }

    /// Add Neo4j lab plugins to get started with the database.
    #[must_use]
    pub fn with_neo4j_labs_plugin(mut self, plugins: &[Neo4jLabsPlugin]) -> Self {
        self.plugins.extend_from_slice(plugins);
        self
    }

    /// Create a new instance of a Neo4j image of the given version with the default user and password.
    ///
    /// # Panics
    ///
    /// If the version is not valid according to the format described in [`Self::with_version()`].
    #[deprecated(since = "0.2.0", note = "Use `from_env().with_version()` instead.")]
    #[must_use]
    pub fn from_version(version: &str) -> Self {
        Self::from_env().with_version(version).unwrap()
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
    pub fn from_auth_and_version(version: &str, user: &str, pass: &str) -> Self {
        Self::from_env()
            .with_version(version)
            .unwrap()
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
    Unset,
}

struct ValidateVersion(bool);

impl<'a> lenient_semver::VersionBuilder<'a> for ValidateVersion {
    type Out = bool;

    fn new() -> Self {
        Self(true)
    }

    fn build(self) -> Self::Out {
        self.0
    }

    fn add_additional(&mut self, _num: u64) {
        self.0 = false;
    }

    fn add_pre_release(&mut self, _pre_release: &'a str) {
        self.0 = false;
    }

    fn add_build(&mut self, _build: &'a str) {
        self.0 = false;
    }
}

impl Default for Neo4j {
    fn default() -> Self {
        Self::from_env()
    }
}

/// The actual Neo4j testcontainers image type which is returned by `container.image()`
pub struct Neo4jImage {
    version: String,
    auth: Option<(String, String)>,
    env_vars: HashMap<String, String>,
    state: RefCell<Option<ContainerState>>,
}

impl Neo4jImage {
    /// Return the version of the Neo4j image.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Return the user/password authentication tuple of the Neo4j server.
    /// If no authentication is set, `None` is returned.
    #[must_use]
    pub fn auth(&self) -> Option<(&str, &str)> {
        self.auth
            .as_ref()
            .map(|(user, pass)| (user.as_str(), pass.as_str()))
    }

    /// Return the user of the Neo4j server.
    /// If no authentication is set, `None` is returned.
    #[must_use]
    pub fn user(&self) -> Option<&str> {
        self.auth().map(|(user, _)| user)
    }

    /// Return the password of the Neo4j server.
    /// If no authentication is set, `None` is returned.
    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.auth().map(|(_, pass)| pass)
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
    fn enterprise_env(&self) -> impl IntoIterator<Item = (String, String)> {
        self.enterprise.then(|| {
            (
                "NEO4J_ACCEPT_LICENSE_AGREEMENT".to_owned(),
                "yes".to_owned(),
            )
        })
    }

    fn auth_env(&self) -> impl IntoIterator<Item = (String, String)> {
        fn auth(image: &Neo4j) -> Option<String> {
            let user = Neo4j::value(&image.user)?;
            let pass = Neo4j::value(&image.pass)?;
            Some(format!("{}/{}", user, pass))
        }

        let auth = auth(self).unwrap_or_else(|| "none".to_owned());
        Some(("NEO4J_AUTH".to_owned(), auth))
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
        let pass = Self::value(&self.pass)?;

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

        for (key, value) in self.enterprise_env() {
            env_vars.insert(key, value);
        }

        for (key, value) in self.auth_env() {
            env_vars.insert(key, value);
        }

        for (key, value) in self.plugins_env() {
            env_vars.insert(key, value);
        }

        for (key, value) in self.conf_env() {
            env_vars.insert(key, value);
        }

        let auth = Self::value(&self.user).and_then(|user| {
            Self::value(&self.pass).map(|pass| (user.into_owned(), pass.into_owned()))
        });

        let version = Self::value(&self.version).expect("Version must be set");
        let version = format!(
            "{}{}",
            version,
            if self.enterprise { "-enterprise" } else { "" }
        );

        Neo4jImage {
            version,
            auth,
            env_vars,
            state: RefCell::new(None),
        }
    }

    fn value(value: &Value) -> Option<Cow<'_, str>> {
        Some(match value {
            &Value::Env { var, fallback } => {
                std::env::var(var).map_or_else(|_| fallback.into(), Into::into)
            }
            &Value::Default(value) => value.into(),
            Value::Value(value) => value.as_str().into(),
            Value::Unset => return None,
        })
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
            .field("auth", &self.auth())
            .field("env_vars", &self.env_vars)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_valid_version() {
        let neo4j = Neo4j::new().with_version("4.2.0").unwrap().build();
        assert_eq!(neo4j.version, "4.2.0");
    }

    #[test]
    fn set_partial_version() {
        let neo4j = Neo4j::new().with_version("4.2").unwrap().build();
        assert_eq!(neo4j.version, "4.2");

        let neo4j = Neo4j::new().with_version("4").unwrap().build();
        assert_eq!(neo4j.version, "4");
    }

    #[test]
    fn set_enterprise_version() {
        let msg = Neo4j::new()
            .with_version("4.2.0-enterprise")
            .unwrap_err()
            .to_string();
        assert_eq!(msg, "Invalid version: 4.2.0-enterprise");
    }

    #[test]
    fn set_invalid_version() {
        let msg = Neo4j::new()
            .with_version("lorem ipsum")
            .unwrap_err()
            .to_string();
        assert_eq!(msg, "Invalid version: lorem ipsum");
    }

    #[test]
    fn set_user() {
        let neo4j = Neo4j::new().with_user("Benutzer").build();
        assert_eq!(neo4j.user(), Some("Benutzer"));
        assert_eq!(neo4j.auth(), Some(("Benutzer", "neo")));
        assert_eq!(neo4j.env_vars.get("NEO4J_AUTH").unwrap(), "Benutzer/neo");
    }

    #[test]
    fn set_password() {
        let neo4j = Neo4j::new().with_password("Passwort").build();
        assert_eq!(neo4j.password(), Some("Passwort"));
        assert_eq!(neo4j.auth(), Some(("neo4j", "Passwort")));
        assert_eq!(neo4j.env_vars.get("NEO4J_AUTH").unwrap(), "neo4j/Passwort");
    }

    #[test]
    fn set_short_password() {
        let neo4j = Neo4j::new().with_password("1337").build();
        assert_eq!(neo4j.password(), Some("1337"));
        assert_eq!(neo4j.auth(), Some(("neo4j", "1337")));
        assert_eq!(
            neo4j
                .env_vars
                .get("NEO4J_dbms_security_auth__minimum__password__length")
                .unwrap(),
            "4"
        );
    }

    #[test]
    fn disable_auth() {
        let neo4j = Neo4j::new().without_authentication().build();
        assert_eq!(neo4j.password(), None);
        assert_eq!(neo4j.user(), None);
        assert_eq!(neo4j.auth(), None);
        assert_eq!(neo4j.env_vars.get("NEO4J_AUTH").unwrap(), "none");
    }

    #[test]
    fn single_plugin_definition() {
        let neo4j = Neo4j::new()
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc])
            .build();
        assert_eq!(
            neo4j.env_vars.get("NEO4JLABS_PLUGINS").unwrap(),
            "[\"apoc\"]"
        );
    }

    #[test]
    fn multiple_plugin_definition() {
        let neo4j = Neo4j::new()
            .with_neo4j_labs_plugin(&[Neo4jLabsPlugin::Apoc, Neo4jLabsPlugin::Bloom])
            .build();
        assert_eq!(
            neo4j.env_vars.get("NEO4JLABS_PLUGINS").unwrap(),
            "[\"apoc\",\"bloom\"]"
        );
    }

    #[test]
    fn multiple_wiht_plugin_calls() {
        let neo4j = Neo4j::new()
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
