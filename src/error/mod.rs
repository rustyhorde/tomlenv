// Copyright (c) 2018 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` errors
mod codes;
mod sources;

pub(crate) use codes::ErrCode;
use getset::Getters;
pub(crate) use sources::ErrSource;
use std::fmt;

/// A result that must include an `tomlenv::Error`
pub type Result<T> = std::result::Result<T, Error>;

/// An error from the library
#[derive(Debug, Getters)]
#[getset(get = "pub(crate)")]
#[allow(dead_code)]
pub struct Error {
    /// the code
    code: ErrCode,
    /// the reason
    reason: String,
    /// the description
    description: String,
    /// the source
    source: Option<ErrSource>,
}

impl Error {
    pub(crate) fn new<U>(code: ErrCode, reason: U, source: Option<ErrSource>) -> Self
    where
        U: Into<String>,
    {
        let reason = reason.into();
        let code_str: &str = code.into();
        let description = format!("{code_str}: {reason}");

        Self {
            code,
            reason,
            description,
            source,
        }
    }

    /// Generate an invalid runtime environment error
    #[must_use]
    pub fn invalid_runtime_environment(env: &str) -> Self {
        Self::new(
            ErrCode::Env,
            format!("invalid runtime environment '{env}'"),
            None,
        )
    }

    pub(crate) fn invalid_current_environment(var: &str) -> Self {
        Self::new(
            ErrCode::Env,
            format!("invalid current environment '{var}'"),
            None,
        )
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(ref x) = self.source {
            Some(x)
        } else {
            None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl From<&str> for Error {
    fn from(text: &str) -> Self {
        let split = text.split(':');
        let vec = split.collect::<Vec<&str>>();
        let code = vec.first().unwrap_or(&"");
        let reason = vec.get(1).unwrap_or(&"");
        Self::new((*code).into(), *reason, None)
    }
}

impl From<String> for Error {
    fn from(text: String) -> Self {
        let split = text.split(':');
        let vec = split.collect::<Vec<&str>>();
        let code = vec.first().unwrap_or(&"");
        let reason = vec.get(1).unwrap_or(&"");
        Self::new((*code).into(), *reason, None)
    }
}

// impl<S> From<<S as TryFrom<String>>::Error> for Error
// where
//     S: DeserializeOwned + Serialize + Ord + PartialOrd + TryFrom<String>,
// {
//     fn from(_s: S) -> Self {
//         Self::new(ErrCode::Env, "", None)
//     }
// }
