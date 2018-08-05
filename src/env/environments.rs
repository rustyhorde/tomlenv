// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` environments configuration
use error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

/// Hold environment specific data as a map from your environment hierarchy key to data struct
/// containg the config for that particular environment.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate getset;
/// # #[macro_use] extern crate serde_derive;
/// # extern crate tomlenv;
/// #
/// # use tomlenv::{Environment, Environments, Result};
/// # use std::env;
/// # use std::io::Cursor;
/// #
/// # fn foo() -> Result<()> {
/// // Your environment specific data struct
/// // *NOTE*: This must implement `Deserialize` and `Serialize`
/// #[derive(Debug, Deserialize, Getters, Serialize)]
/// struct RuntimeEnv {
///   #[get]
///   name: String,
///   #[get]
///   key: Option<String>,
/// }
///
/// // Your environment specific configuration
/// let toml = r#"[envs.prod]
/// name = "Production"
/// key = "abcd-123-efg-45"
///
/// [envs.stage]
/// name = "Stage"
///
/// [envs.test]
/// name = "Test"
///
/// [envs.dev]
/// name = "Development"
///
/// [envs.local]
/// name = "Local"
/// "#;
///
/// // Deserialize the TOML config into your environment structs
/// let mut buffer = String::new();
/// let mut cursor = Cursor::new(toml);
/// let envs: Environments<Environment, RuntimeEnv> = Environments::from_reader(&mut cursor, &mut buffer)?;
///
/// // Test that all the environments are present
/// env::set_var("env", "prod");
/// let mut current = envs.current()?;
/// assert_eq!(current.name(), "Production");
/// assert_eq!(current.key(), &Some("abcd-123-efg-45".to_string()));
///
/// env::set_var("env", "stage");
/// current = envs.current()?;
/// assert_eq!(current.name(), "Stage");
/// assert_eq!(current.key(), &None);
///
/// env::set_var("env", "test");
/// current = envs.current()?;
/// assert_eq!(current.name(), "Test");
/// assert_eq!(current.key(), &None);
///
/// env::set_var("env", "dev");
/// current = envs.current()?;
/// assert_eq!(current.name(), "Development");
/// assert_eq!(current.key(), &None);
///
/// env::set_var("env", "local");
/// current = envs.current()?;
/// assert_eq!(current.name(), "Local");
/// assert_eq!(current.key(), &None);
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct Environments<S, T>
where
    S: Ord,
{
    /// A map of `Environment` to struct
    envs: BTreeMap<S, T>,
}

impl<'de, S, T> Environments<S, T>
where
    T: Deserialize<'de> + Serialize,
    S: Deserialize<'de> + Serialize + Ord + PartialOrd + TryFrom<String>,
    Error: From<<S as TryFrom<String>>::Error>,
{
    /// Load the environments from a path.
    pub fn from_path(path: &'de Path, buffer: &'de mut String) -> Result<Self> {
        let mut file = File::open(path)?;
        file.read_to_string(buffer)?;
        let env = toml::from_str(buffer)?;
        Ok(env)
    }

    /// Load the environments from a reader.
    pub fn from_reader<R>(reader: &'de mut R, buffer: &'de mut String) -> Result<Self>
    where
        R: Read,
    {
        reader.read_to_string(buffer)?;
        let env = toml::from_str(buffer)?;
        Ok(env)
    }

    /// Get the current environment
    pub fn current(&self) -> Result<&T> {
        let environment = TryFrom::try_from(env::var("env")?)?;
        Ok(self
            .envs
            .get(&environment)
            .ok_or_else(|| "Could not get current environment!")?)
    }
}

#[cfg(test)]
mod test {
    use super::Environments;
    use env::Environment;
    use error::Result;
    use std::collections::BTreeMap;
    use std::env;
    use std::io::Cursor;
    use toml;

    const EXPECTED_TOML_STR: &str = r#"[envs.prod]
name = "Production"
key = "abcd-123-efg-45"

[envs.stage]
name = "Stage"

[envs.test]
name = "Test"

[envs.dev]
name = "Development"

[envs.local]
name = "Local"
"#;

    #[derive(Debug, Deserialize, Getters, Serialize)]
    struct RuntimeEnv {
        #[get]
        name: String,
        #[get]
        key: Option<String>,
    }

    fn try_decode(toml: &str) -> Result<Environments<Environment, RuntimeEnv>> {
        let mut buffer = String::new();
        let mut cursor = Cursor::new(toml);
        Ok(Environments::from_reader(&mut cursor, &mut buffer)?)
    }

    fn try_encode(environments: &Environments<Environment, RuntimeEnv>) -> Result<String> {
        Ok(toml::to_string(environments)?)
    }

    fn try_current(envs: &Environments<Environment, RuntimeEnv>, expected: &str) -> Result<()> {
        let current = envs.current()?;
        assert_eq!(current.name(), expected);
        Ok(())
    }

    #[test]
    fn decode() {
        match try_decode(EXPECTED_TOML_STR) {
            Ok(_) => assert!(true, "Successfully decode TOML to Environments"),
            Err(_) => assert!(false, "Unable to decode TOML to Environments!"),
        }
    }

    #[test]
    fn encode() {
        let mut envs = BTreeMap::new();
        let prod = RuntimeEnv {
            name: "Production".to_string(),
            key: Some("abcd-123-efg-45".to_string()),
        };
        let stage = RuntimeEnv {
            name: "Stage".to_string(),
            key: None,
        };
        let test = RuntimeEnv {
            name: "Test".to_string(),
            key: None,
        };
        let dev = RuntimeEnv {
            name: "Development".to_string(),
            key: None,
        };
        let local = RuntimeEnv {
            name: "Local".to_string(),
            key: None,
        };
        envs.insert(Environment::Prod, prod);
        envs.insert(Environment::Stage, stage);
        envs.insert(Environment::Test, test);
        envs.insert(Environment::Dev, dev);
        envs.insert(Environment::Local, local);

        let environments = Environments { envs: envs };

        match try_encode(&environments) {
            Ok(toml) => assert_eq!(toml, EXPECTED_TOML_STR, "TOML strings match"),
            Err(_) => assert!(false, "Unable to encode Environments to TOML"),
        }
    }

    #[test]
    fn current() {
        match try_decode(EXPECTED_TOML_STR) {
            Ok(ref envs) => {
                env::set_var("env", "prod");
                match try_current(envs, "Production") {
                    Ok(_) => assert!(true, "Found Production Env"),
                    Err(_) => assert!(false, "Current is not Production!"),
                }
                env::set_var("env", "stage");
                match try_current(envs, "Stage") {
                    Ok(_) => assert!(true, "Found Stage Env"),
                    Err(_) => assert!(false, "Current is not Stage!"),
                }
                env::set_var("env", "test");
                match try_current(envs, "Test") {
                    Ok(_) => assert!(true, "Found Test Env"),
                    Err(_) => assert!(false, "Current is not Test!"),
                }
                env::set_var("env", "dev");
                match try_current(envs, "Development") {
                    Ok(_) => assert!(true, "Found Development Env"),
                    Err(_) => assert!(false, "Current is not Development!"),
                }
                env::set_var("env", "local");
                match try_current(envs, "Local") {
                    Ok(_) => assert!(true, "Found Local Env"),
                    Err(_) => assert!(false, "Current is not Local!"),
                }
            }
            Err(_) => assert!(false, "Unable to decode TOML to Environments!"),
        }
    }
}
