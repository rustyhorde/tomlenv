// Copyright (c) 2018 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` default environment hierarchy implementation.
use crate::error::{Error, Result};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;

/// A fairly standard environment hierarchy for use with `Environments`.
/// Prod -> Stage -> Test -> Dev -> Local
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Environment {
    /// Production
    Prod,
    /// Stage
    Stage,
    /// Test
    Test,
    /// Development
    Dev,
    /// Local
    Local,
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnvironmentVisitor;

        impl de::Visitor<'_> for EnvironmentVisitor {
            type Value = Environment;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("any valid environment")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Environment, E>
            where
                E: de::Error,
            {
                TryFrom::try_from(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_string(EnvironmentVisitor)
    }
}

impl Serialize for Environment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let env = match *self {
            Environment::Prod => "prod",
            Environment::Stage => "stage",
            Environment::Test => "test",
            Environment::Dev => "dev",
            Environment::Local => "local",
        };
        write!(f, "{env}")
    }
}

impl TryFrom<&str> for Environment {
    type Error = Error;

    fn try_from(env: &str) -> Result<Self> {
        match env {
            "prod" => Ok(Environment::Prod),
            "stage" => Ok(Environment::Stage),
            "test" => Ok(Environment::Test),
            "dev" => Ok(Environment::Dev),
            "local" => Ok(Environment::Local),
            _ => Err(Error::invalid_runtime_environment(env)),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = Error;

    fn try_from(env: String) -> Result<Self> {
        match &env[..] {
            "prod" => Ok(Environment::Prod),
            "stage" => Ok(Environment::Stage),
            "test" => Ok(Environment::Test),
            "dev" => Ok(Environment::Dev),
            "local" => Ok(Environment::Local),
            _ => Err(Error::invalid_runtime_environment(&env)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Environment;
    use std::convert::TryFrom;

    #[test]
    fn display() {
        assert_eq!(Environment::Prod.to_string(), "prod");
        assert_eq!(Environment::Stage.to_string(), "stage");
        assert_eq!(Environment::Test.to_string(), "test");
        assert_eq!(Environment::Dev.to_string(), "dev");
        assert_eq!(Environment::Local.to_string(), "local");
    }

    #[test]
    fn convert() {
        match Environment::try_from("prod") {
            Ok(re) => assert_eq!(re, Environment::Prod),
            Err(_) => assert!(false, "Invalid 'prod' Runtime Environment"),
        }
        match Environment::try_from("stage") {
            Ok(re) => assert_eq!(re, Environment::Stage),
            Err(_) => assert!(false, "Invalid 'stage' Runtime Environment"),
        }
        match Environment::try_from("test") {
            Ok(re) => assert_eq!(re, Environment::Test),
            Err(_) => assert!(false, "Invalid 'test' Runtime Environment"),
        }
        match Environment::try_from("dev") {
            Ok(re) => assert_eq!(re, Environment::Dev),
            Err(_) => assert!(false, "Invalid 'dev' Runtime Environment"),
        }
        match Environment::try_from("blah") {
            Ok(_) => assert!(false, "'blah' is not a good runtime environment!"),
            Err(_) => assert!(true, "'blah' failed to convert properly"),
        }
    }
}
