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

mod logging;
mod macros;
pub mod merge;
mod server;
mod smtp;

use crate::{templates, var};
use eyre::Report;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use self::merge::Merge;

/// Infailliable type to map system environment variables into Rust types easily.
pub trait FromEnv {
    /// Output type.
    type Output;

    fn from_env() -> Self::Output;
}

/// Failliable type to map system environment variables into Rust types easily.
pub trait TryFromEnv {
    /// Output type.
    type Output;

    /// Error variant.
    type Err;

    fn try_from_env() -> Result<Self::Output, Self::Err>;
}

impl<O, T: FromEnv<Output = O>> TryFromEnv for T {
    type Output = O;
    type Err = Infallible;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        Ok(Self::from_env())
    }
}

/// Represents the configuration type that is used to map a YAML file or the system
/// environment variables into this struct here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Resolvable URL to a Sentry DSN that can be used to capture errors
    /// when they occur through Sentry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentry_dsn: Option<String>,

    /// Configuration to resolve templates from.
    #[serde(default, with = "serde_yaml::with::singleton_map")]
    pub templates: templates::Config,

    /// Configuration for the logging system.
    #[serde(default)]
    pub logging: logging::Config,

    /// Server configuration.
    #[serde(default)]
    pub server: server::Config,

    /// Configuration to connect to a SMTP server.
    #[serde(default)]
    pub smtp: smtp::Config,
}

impl TryFromEnv for Config {
    type Output = Config;
    type Err = Report;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        Ok(Config {
            sentry_dsn: var!("EMAILS_SENTRY_DSN", is_optional: true),
            templates: templates::Config::try_from_env()?,
            logging: logging::Config::try_from_env()?,
            server: server::Config::try_from_env()?,
            smtp: smtp::Config::try_from_env()?,
        })
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        self.sentry_dsn.merge(other.sentry_dsn);
        self.logging.merge(other.logging);
        self.server.merge(other.server);
        self.smtp.merge(other.smtp);
    }
}
