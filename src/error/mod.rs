// Copyright (c) 2018 tomlenv developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tomlenv` errors

/// `tomlenv` errors
#[derive(Debug, Fail)]
pub enum Error {
    /// Generated when try_from cannot convert a string to a runtime environment.
    #[fail(display = "the given runtime environment '{}' is invalid!", env)]
    InvalidRuntimeEnvironment {
        /// The `env` string that could not be converted into a runtime environment.
        env: String,
    },
    /// Generated when the current environment cannot be retrieved from the environments map.
    #[fail(display = "could not retrieve current environment!")]
    InvalidCurrentEnvironment,
}
