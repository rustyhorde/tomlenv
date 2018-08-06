// Copyright (c) 2018 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` allows you to drive your environment configuration from TOML.
//! The `Environments` struct holds a reference from your environment hierarchy
//! to the configuraion associated with that particular enviornment i.e.
//!
//! * prod_key -> prod_config
//! * stage_key -> stage_config
//! * dev_key -> dev_config
//!
//! On the key side, you can use the `Environment` hierarchy defined by this
//! library (Prod -> Stage -> Test -> Dev -> Local), or you can define your own
//! custom hierarchy to use with the `Environments` struct.  If your define a
//! custom hierarchy you must implement the `Deserialize`, `Serialize`, `Ord`,
//! `PartialOrd`, and `TryFrom<String>` traits.  See more in the
//! [Custom Environment Hierarchy](#custom-environment-hierarchy) section below.
//!
//! # Usage
//! First, define a struct that represents your environment configuration.  For
//! items that appear in one environment, but not another, use `Option`.  See
//! the key field in the example below.
//!
//! Second, generate a `Reader` from your TOML.  Normally, the TOML would be
//! defined as a `Path`, and used with the `from_path` method.  You can also
//! supply a generic `Read` implementation to the `from_reader` method, as
//! below.
//!
//! Third, deserialize the TOML into your `Environments` struct.  At this point,
//! you can use the `current` method to access your environment config for the
//! environment specified by the environment variable `env`.
//!
//! ```
//! # #[macro_use] extern crate getset;
//! # #[macro_use] extern crate serde_derive;
//! # extern crate tomlenv;
//! #
//! # use tomlenv::{Environment, Environments, Result};
//! # use std::env;
//! # use std::io::Cursor;
//! #
//! # fn foo() -> Result<()> {
//! /// Define your environment specific configuration.
//! /// *NOTE*: This must implement `Deserialize` and `Serialize`
//! #[derive(Debug, Deserialize, Getters, Serialize)]
//! struct MyAppEnv {
//!   /// The display name of this environment.
//!   #[get]
//!   name: String,
//!   /// The secret key only used in the Prod environment.
//!   #[get]
//!   key: Option<String>,
//! }
//!
//! /// Grab your environment TOML.  This would usually be in a file and can
//! /// be read to a string such as below.
//! let toml = r#"[envs.prod]
//! name = "Production"
//! key = "abcd-123-efg-45"
//!
//! [envs.stage]
//! name = "Stage"
//!
//! [envs.test]
//! name = "Test"
//!
//! [envs.dev]
//! name = "Development"
//!
//! [envs.local]
//! name = "Local"
//! "#;
//!
//! // Deserialize the TOML config into your environment structs.  This example
//! // is using the `Enrivorment` hierarchy supplied by the library.
//! let mut buffer = String::new();
//! let mut cursor = Cursor::new(toml);
//! let envs: Environments<Environment, MyAppEnv> = Environments::from_reader(&mut cursor, &mut buffer)?;
//!
//! // Check the `Production` environment.
//! env::set_var("env", "prod");
//! let mut current = envs.current()?;
//! assert_eq!(current.name(), "Production");
//! assert_eq!(current.key(), &Some("abcd-123-efg-45".to_string()));
//!
//! // Switch to the `Development` environment.
//! env::set_var("env", "dev");
//! current = envs.current()?;
//! assert_eq!(current.name(), "Development");
//! assert_eq!(current.key(), &None);
//! #   Ok(())
//! # }
//! ```
//!
//! # Custom Environment Hierarchy
//! If you wish to forego using the `Environment` hierarchy supplied by this
//! library, implement a custom hierarchy instead.  There are a few traits you
//! must implement in order to work with `Environments`.
//!
//! ## Required
//! * `Deserialize` and `Serialize`: These are required to translate
//! to/from TOML.
//! * `Ord` and `PartialOrd`:  These are required to maintain proper ordering for
//! your hierarchy and ensure serialized TOML is always in the same order.
//! * `TryFrom<String>`: This is used to translate the environment variable `env`
//! into your hierarchy type.
//!
//! ## Optional
//! * `Display`: Used to convert your hierarchy type to a formatted string.
//! Useful if you need to show what environment you are using.
//! * `TryFrom<&'a str>`: In this case, used by the custom deserializer.
//!
//! Below is an example of a custom hierarchy.  This example has a custom
//! serializer/deserializer, but that shouldn't be necessary in all cases.
//!
//! ```
//! # #![feature(try_from)]
//! #
//! # #[macro_use] extern crate getset;
//! # #[macro_use] extern crate serde_derive;
//! #
//! # extern crate serde;
//! # extern crate tomlenv;
//! #
//! # use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
//! # use std::convert::TryFrom;
//! # use std::env;
//! # use std::fmt;
//! # use std::io::Cursor;
//! # use tomlenv::{Environments, Error, Result};
//! #
//! #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
//! pub enum MyHierarchy {
//!     /// Production
//!     Prod,
//!     /// Certification
//!     Cert,
//!     /// Sandbox
//!     Sandbox,
//!     /// Local
//!     Local,
//! }
//!
//! impl fmt::Display for MyHierarchy {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         let env = match *self {
//!             MyHierarchy::Prod => "prod",
//!             MyHierarchy::Cert => "ce",
//!             MyHierarchy::Sandbox => "sb",
//!             MyHierarchy::Local => "local",
//!         };
//!         write!(f, "{}", env)
//!     }
//! }
//!
//! impl<'a> TryFrom<&'a str> for MyHierarchy {
//!     type Error = Error;
//!
//!     fn try_from(env: &str) -> Result<Self> {
//!         match env {
//!             "prod" => Ok(MyHierarchy::Prod),
//!             "ce" => Ok(MyHierarchy::Cert),
//!             "sb" => Ok(MyHierarchy::Sandbox),
//!             "local" => Ok(MyHierarchy::Local),
//!             _ => Err("invalid runtime enviroment".into()),
//!         }
//!     }
//! }
//!
//! impl TryFrom<String> for MyHierarchy {
//!     type Error = Error;
//!
//!     fn try_from(env: String) -> Result<Self> {
//!         match &env[..] {
//!             "prod" => Ok(MyHierarchy::Prod),
//!             "ce" => Ok(MyHierarchy::Cert),
//!             "sb" => Ok(MyHierarchy::Sandbox),
//!             "local" => Ok(MyHierarchy::Local),
//!             _ => Err("invalid runtime enviroment".into()),
//!         }
//!     }
//! }
//!
//! impl Serialize for MyHierarchy {
//!     fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         serializer.serialize_str(&self.to_string())
//!     }
//! }
//!
//! impl<'de> Deserialize<'de> for MyHierarchy {
//!     fn deserialize<D>(deserializer: D) -> ::std::result::Result<MyHierarchy, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         struct MyHierarchyVisitor;
//!
//!         impl<'de> de::Visitor<'de> for MyHierarchyVisitor {
//!             type Value = MyHierarchy;
//!
//!             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//!                 formatter.write_str("any valid environment")
//!             }
//!
//!             fn visit_str<E>(self, value: &str) -> ::std::result::Result<MyHierarchy, E>
//!             where
//!                 E: de::Error,
//!             {
//!                 TryFrom::try_from(value).map_err(de::Error::custom)
//!             }
//!         }
//!
//!         deserializer.deserialize_string(MyHierarchyVisitor)
//!     }
//! }
//!
//! #[derive(Debug, Deserialize, Getters, Serialize)]
//! struct MyAppEnv {
//!   /// The display name of this environment.
//!   #[get]
//!   name: String,
//!   /// The secret key only used in the Prod environment.
//!   #[get]
//!   key: Option<String>,
//! }
//! #
//! # fn foo() -> Result<()> {
//!
//! /// Grab your environment TOML.  This would usually be in a file and can
//! /// be read to a string such as below.
//! let toml = r#"[envs.prod]
//! name = "Production"
//! key = "abcd-123-efg-45"
//!
//! [envs.ce]
//! name = "Certification"
//!
//! [envs.sb]
//! name = "Sandbox"
//!
//! [envs.local]
//! name = "Local"
//! "#;
//!
//! // Deserialize the TOML config into your environment structs.  This example
//! // is using the custom `MyHierarchy` hierarchy.
//! let mut buffer = String::new();
//! let mut cursor = Cursor::new(toml);
//! let envs: Environments<MyHierarchy, MyAppEnv> = Environments::from_reader(&mut cursor, &mut buffer)?;
//!
//! // Check the `Production` environment.
//! env::set_var("env", "prod");
//! let mut current = envs.current()?;
//! assert_eq!(current.name(), "Production");
//! assert_eq!(current.key(), &Some("abcd-123-efg-45".to_string()));
//!
//! // Switch to the `Sandbox` environment.
//! env::set_var("env", "sb");
//! current = envs.current()?;
//! assert_eq!(current.name(), "Sandbox");
//! assert_eq!(current.key(), &None);
//! #   Ok(())
//! # }
//! ```
#![feature(try_from)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate error_chain;
#[cfg(test)]
#[macro_use]
extern crate getset;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate toml;

mod env;
mod error;

pub use env::{Environment, Environments};
pub use error::{Error, ErrorKind, Result};
