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

use std::net::SocketAddr;

use crate::var;

use super::{merge::Merge, TryFromEnv};
use eyre::Report;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use tracing::Level;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// URL to a TCP server that can collect the JSON messages that are emitted. This must
    /// be established before the gRPC server starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logstash_write_url: Option<String>,

    /// Whether or not JSON messages are emitted rather than the default, prettified
    /// logging system. This is useful if you need to `jq` the messages.
    #[serde(default)]
    pub json: bool,

    /// Configures the logging level that is used to emit log information.
    #[serde(
        default = "default_level",
        serialize_with = "serialize_level",
        deserialize_with = "deserialize_level "
    )]
    pub level: Level,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logstash_write_url: None,
            level: default_level(),
            json: false,
        }
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Err = Report;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        let level = var!("EMAILS_LOG_LEVEL", to: Level, or_else: Level::INFO);

        // check if the logstash url is a valid SocketAddr
        if let Some(addr) = var!("EMAILS_LOGSTASH_TCP_URL", is_optional: true) {
            addr.parse::<SocketAddr>()
                .map_err(|e| eyre!("unable to parse address [{addr}]: {e}"))?;
        }

        Ok(Config {
            logstash_write_url: var!("EMAILS_LOGSTASH_TCP_URL", is_optional: true),
            level,
            json: var!("EMAILS_LOG_JSON", to: bool, or_else: false),
        })
    }
}

impl Merge for Level {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        self.logstash_write_url.merge(other.logstash_write_url);
        self.level.merge(other.level);
        self.json.merge(other.json);
    }
}

#[inline(always)]
const fn default_level() -> Level {
    Level::INFO
}

/// [`Serializer`] implementation for [`Level`].
fn serialize_level<S: Serializer>(filter: &Level, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(match *filter {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::ERROR => "error",
        Level::WARN => "warn",
        Level::INFO => "info",
    })
}

/// [`Deserializer`] implementation for [`Level`].
fn deserialize_level<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Level, D::Error> {
    let string = String::deserialize(deserializer)?;
    match string.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "error" => Ok(Level::ERROR),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        level => Err(D::Error::custom(format!(
            "wanted [trace, debug, error, info, warn]; received {level} instead"
        ))),
    }
}
