// 🐻‍❄️💌 email-service: charted's email service built in Rust that can be connected via gRPC
// Copyright 2023 Noelware, LLC. <team@noelware.org>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{error::Error as _, net::AddrParseError, path::PathBuf};

use tonic::Status;

#[macro_export]
macro_rules! to_dyn_error {
    ($e:expr) => {
        $crate::error::Error::Unknown(Box::new($e))
    };
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lettre error: {0}")]
    Lettre(#[from] lettre::error::Error),

    #[error("Mustache error: {0}")]
    Mustache(#[from] mustache::Error),

    #[error("{0}")]
    Unknown(#[from] Box<dyn std::error::Error>),

    #[error("YAML serialization: {0}")]
    YamlSerialization(#[from] serde_yaml::Error),

    #[error("gRPC error ({}): {}", status.code(), status.message())]
    Grpc { status: Status },

    #[error("Level {level} is not a valid log level")]
    UnknownLogLevel { level: String },

    #[error("Unable to parse [{value}] as a socket address: {}", error.description())]
    ParseSocketError {
        #[source]
        error: AddrParseError,
        value: String,
    },

    #[error("Template [{template:?}] was not found")]
    TemplateNotFound { template: PathBuf },
}
