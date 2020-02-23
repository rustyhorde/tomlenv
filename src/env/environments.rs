// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` environments configuration
use crate::error::{Error, Result};
use clap::ArgMatches;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use toml;

/// Hold environment specific data as a map from your environment hierarchy key to data struct
/// containg the config for that particular environment.
///
/// # Example
///
/// ```
/// # use tomlenv::{Environment, Environments, Error, Result};
/// # use std::env;
/// # use std::io::Cursor;
/// # use serde_derive::{Deserialize, Serialize};
/// # use getset::Getters;
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
/// let mut cursor = Cursor::new(toml);
/// let envs: Environments<Environment, RuntimeEnv> = Environments::from_reader(&mut cursor)?;
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
#[derive(Clone, Debug, De, Ser)]
pub struct Environments<S, T>
where
    S: Ord,
{
    /// A map of `Environment` to struct
    envs: BTreeMap<S, T>,
}

impl<S, T> Environments<S, T>
where
    T: DeserializeOwned + Serialize,
    S: DeserializeOwned + Serialize + Ord + PartialOrd + TryFrom<String>,
{
    /// Load the environments from a path.
    pub fn from_path(path: &Path) -> Result<Self> {
        match File::open(path) {
            Ok(mut file) => {
                let mut buffer = String::new();
                let _ = file.read_to_string(&mut buffer)?;
                Ok(toml::from_str(&buffer)?)
            }
            Err(e) => {
                eprintln!("Unable to read '{}'", path.display());
                Err(e.into())
            }
        }
    }

    /// Load the environments from a reader.
    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: Read,
    {
        let mut buffer = String::new();
        let _ = reader.read_to_string(&mut buffer)?;
        Ok(toml::from_str(&buffer)?)
    }

    /// Get the current environment
    pub fn current(&self) -> Result<&T> {
        self.current_from("env")
    }

    /// Get the current environment from the given variable
    pub fn current_from(&self, var: &'static str) -> Result<&T> {
        let environment = TryFrom::try_from(env::var(var)?)
            .map_err(|_e| Error::invalid_current_environment(var))?;
        Ok(self
            .envs
            .get(&environment)
            .ok_or_else(|| Error::invalid_current_environment(var))?)
    }
}

impl<'a, S, T> TryFrom<&'a ArgMatches<'a>> for Environments<S, T>
where
    T: DeserializeOwned + Serialize,
    S: DeserializeOwned + Serialize + Ord + PartialOrd + TryFrom<String>,
{
    type Error = Error;

    fn try_from(matches: &'a ArgMatches<'a>) -> Result<Self> {
        let env_path = if let Some(env_path) = matches.value_of("env_path") {
            PathBuf::from(env_path).join("env.toml")
        } else {
            PathBuf::from("env.toml")
        };

        Ok(Environments::from_path(env_path.as_path())?)
    }
}

#[cfg(test)]
mod test {
    use super::Environments;
    use crate::{env::Environment, error::Result};
    use clap::{App, Arg};
    use dirs;
    use getset::Getters;
    use serde_derive::{Deserialize, Serialize};
    use std::{
        collections::BTreeMap,
        convert::TryFrom,
        env,
        fs::{remove_file, OpenOptions},
        io::{BufWriter, Cursor, Write},
    };
    use toml;

    const TOMLENV: &str = "TOMLENV";
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
        let mut cursor = Cursor::new(toml);
        Ok(Environments::from_reader(&mut cursor)?)
    }

    fn try_encode(environments: &Environments<Environment, RuntimeEnv>) -> Result<String> {
        Ok(toml::to_string(environments)?)
    }

    fn try_current(envs: &Environments<Environment, RuntimeEnv>, expected: &str) -> Result<()> {
        let current = envs.current()?;
        assert_eq!(current.name(), expected);
        Ok(())
    }

    fn try_current_from(
        var: &'static str,
        envs: &Environments<Environment, RuntimeEnv>,
        expected: &str,
    ) -> Result<()> {
        let current = envs.current_from(var)?;
        assert_eq!(current.name(), expected);
        Ok(())
    }

    fn test_cli() -> App<'static, 'static> {
        App::new("env-from-app-matches")
            .version("1")
            .author("Yoda")
            .about("command line for proxy config testing")
            .arg(
                Arg::with_name("env_path")
                    .short("e")
                    .long("envpath")
                    .takes_value(true)
                    .value_name("ENV_PATH"),
            )
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
        let _ = envs.insert(Environment::Prod, prod);
        let _ = envs.insert(Environment::Stage, stage);
        let _ = envs.insert(Environment::Test, test);
        let _ = envs.insert(Environment::Dev, dev);
        let _ = envs.insert(Environment::Local, local);

        let environments = Environments { envs };

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

    #[test]
    fn current_from() {
        match try_decode(EXPECTED_TOML_STR) {
            Ok(ref envs) => {
                env::set_var(TOMLENV, "prod");
                match try_current_from(TOMLENV, envs, "Production") {
                    Ok(_) => assert!(true, "Found Production Env"),
                    Err(_) => assert!(false, "Current is not Production!"),
                }
                env::set_var(TOMLENV, "stage");
                match try_current_from(TOMLENV, envs, "Stage") {
                    Ok(_) => assert!(true, "Found Stage Env"),
                    Err(_) => assert!(false, "Current is not Stage!"),
                }
                env::set_var(TOMLENV, "test");
                match try_current_from(TOMLENV, envs, "Test") {
                    Ok(_) => assert!(true, "Found Test Env"),
                    Err(_) => assert!(false, "Current is not Test!"),
                }
                env::set_var(TOMLENV, "dev");
                match try_current_from(TOMLENV, envs, "Development") {
                    Ok(_) => assert!(true, "Found Development Env"),
                    Err(_) => assert!(false, "Current is not Development!"),
                }
                env::set_var(TOMLENV, "local");
                match try_current_from(TOMLENV, envs, "Local") {
                    Ok(_) => assert!(true, "Found Local Env"),
                    Err(_) => assert!(false, "Current is not Local!"),
                }
            }
            Err(_) => assert!(false, "Unable to decode TOML to Environments!"),
        }
    }

    #[test]
    fn try_from() {
        if let Some(data_local_dir) = dirs::data_local_dir() {
            let env_toml = data_local_dir.join("env.toml");
            if let Ok(tmpfile) = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&env_toml)
            {
                let mut writer = BufWriter::new(tmpfile);
                writer
                    .write_all(EXPECTED_TOML_STR.as_bytes())
                    .expect("Unable to write tmpfile");
            }

            let blah = format!("{}", data_local_dir.display());
            let arg_vec: Vec<&str> = vec!["env-from-app-matches", "--envpath", &blah];
            let matches = test_cli().get_matches_from(arg_vec);
            match Environments::try_from(&matches) {
                Ok(e) => {
                    let _b: Environments<Environment, RuntimeEnv> = e;
                    assert!(true);
                }
                Err(_) => assert!(false, "Unable to deserialize environments"),
            }

            remove_file(env_toml).expect("Unable to remove tmp 'env.toml'");
        }
    }
}
