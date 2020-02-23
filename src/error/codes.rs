// Copyright (c) 2018,2019,2020 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Error Codes
use std::fmt;

/// Error Codes
#[derive(Copy, Clone, Debug)]
crate enum ErrCode {
    /// An error caused by the client
    Client,
    /// An environmental error
    Env,
    /// A framework related error
    Framework,
    /// An error caused by HTTP client
    HttpClient,
    /// An I/O error
    Io,
    /// An error parsing
    Parse,
    /// An error caused by the server
    Server,
    /// Unauthorized to perform the requested actions
    Unauthorized,
    /// An unknown error
    Unknown,
}

impl fmt::Display for ErrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Client => "client",
                Self::Env => "env",
                Self::Framework => "framework",
                Self::HttpClient => "httpclient",
                Self::Io => "io",
                Self::Parse => "parse",
                Self::Server => "server",
                Self::Unauthorized => "unauthorized",
                Self::Unknown => "unknown",
            }
        )
    }
}

impl Into<&str> for ErrCode {
    #[must_use]
    fn into(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Env => "env",
            Self::Framework => "framework",
            Self::HttpClient => "httpclient",
            Self::Io => "io",
            Self::Parse => "parse",
            Self::Server => "server",
            Self::Unauthorized => "unauthorized",
            Self::Unknown => "unknown",
        }
    }
}

impl Into<String> for ErrCode {
    #[must_use]
    fn into(self) -> String {
        let tmp: &str = self.into();
        tmp.to_string()
    }
}

impl From<&str> for ErrCode {
    #[must_use]
    fn from(text: &str) -> Self {
        match text {
            "client" => Self::Client,
            "env" => Self::Env,
            "framework" => Self::Framework,
            "httpclient" => Self::HttpClient,
            "io" => Self::Io,
            "parse" => Self::Parse,
            "server" => Self::Server,
            "unauthorized" => Self::Unauthorized,
            _ => Self::Unknown,
        }
    }
}
