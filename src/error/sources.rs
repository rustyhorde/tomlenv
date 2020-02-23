// Copyright (c) 2018,2019,2020 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Error Sources
use crate::error::{ErrCode, Error};
use std::fmt;

macro_rules! dep_error {
    ($error:ty, $kind:expr, $code:expr, $reason:expr) => {
        impl From<$error> for Error {
            #[must_use]
            fn from(inner: $error) -> Self {
                Self::new($code, $reason, Some($kind(inner)))
            }
        }
    };
}

dep_error!(
    std::env::VarError,
    ErrSource::Var,
    ErrCode::Env,
    "There was an error processing your enviroment"
);
dep_error!(
    std::io::Error,
    ErrSource::Io,
    ErrCode::Io,
    "There was an error processing your request"
);
dep_error!(
    toml::de::Error,
    ErrSource::TomlDe,
    ErrCode::Parse,
    "There was an error deserializing TOML"
);
dep_error!(
    toml::ser::Error,
    ErrSource::TomlSer,
    ErrCode::Parse,
    "There was an error serializing TOML"
);

/// DataQ Error Source
#[derive(Debug)]
#[allow(clippy::large_enum_variant, variant_size_differences)]
crate enum ErrSource {
    /// An I/O error
    Io(std::io::Error),
    /// An error deserializing TOML
    TomlDe(toml::de::Error),
    /// An error serializing TOML
    TomlSer(toml::ser::Error),
    /// An error reading an environment variable
    Var(std::env::VarError),
}

impl std::error::Error for ErrSource {}

impl fmt::Display for ErrSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(f, "{}", source),
            Self::TomlDe(source) => write!(f, "{}", source),
            Self::TomlSer(source) => write!(f, "{}", source),
            Self::Var(source) => write!(f, "{}", source),
        }
    }
}
