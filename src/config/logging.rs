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

use crate::{error::Error, generate_config_struct};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

use std::str::FromStr;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    #[default]
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Off => LevelFilter::Off,
        }
    }
}

impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "off" => Ok(LogLevel::Off),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(Error::UnknownLogLevel {
                level: s.to_owned(),
            }),
        }
    }
}

generate_config_struct!(LogConfig {
    ///
    /// URL to connect to Logstash via TCP that outputs all logs into Logstash.
    #[serde(skip_serializing_if = "Option::is_none")]
    logstash_uri: Option<String> => {
        on_env -> ::std::env::var("EMAILS_LOGSTASH_URI").ok();
        on_default -> None;
    },

    ///
    /// The level to use when configuring the logger. By default, all information logging
    /// will be outputted.
    #[serde(default)]
    pub level: LogLevel => {
        on_env -> ::std::env::var("EMAILS_LOG_LEVEL").map(|p| p.parse::<LogLevel>().expect("unable to parse into log level")).unwrap_or_default();
        on_default -> LogLevel::default();
    },

    ///
    /// If the server should print out JSON logging instead of the default, prettier
    /// logging.
    #[serde(default = "default_json")]
    pub json: bool => {
        on_env -> ::std::env::var("EMAILS_LOG_IN_JSON").map(|f| f.parse::<bool>().expect("Unable to parse into boolean")).unwrap_or(default_json());
        on_default -> default_json();
    }
});

fn default_json() -> bool {
    false
}

impl LogConfig {
    pub fn logstash_uri(&self) -> Option<String> {
        self.logstash_uri.clone()
    }
}
