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

use std::{
    fs::File,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    error::{Error, Result},
    to_dyn_error,
};

use log::LevelFilter;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static CONFIG: OnceCell<Config> = OnceCell::new();

/// Trait for implementing `T` from environment variables. This is used in the
/// `generate_config_struct` macro.
pub trait FromEnv<T> {
    fn from_env() -> T;
}

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

#[macro_export]
macro_rules! generate_config_struct {
    ($name:ident {
        $(
            #[doc = $help:expr]
            $(#[$meta:meta])*
            $vis:vis $key:ident: $ty:ty => {
                on_env -> $env_block:expr;
                on_default -> $default_block:expr;
            }$(,)?
        ),+ $(,)?
    }) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
                $(#[$meta])*
                $vis $key: $ty,
            )*
        }

        impl FromEnv<$name> for $name {
            fn from_env() -> $name {
                $name {
                    $(
                        $key: $env_block,
                    )*
                }
            }
        }

        impl Default for $name {
            fn default() -> $name {
                $name {
                    $(
                        $key: $default_block,
                    )*
                }
            }
        }
    };
}

generate_config_struct!(Config {
    ///
    /// The DSN for using Sentry for tracing and tracking down errors when it happens when you use this service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentry_dsn: Option<String> => {
        on_env -> ::std::env::var("EMAILS_SENTRY_DSN").ok();
        on_default -> None;
    },

    ///
    /// The directory to load up templates from
    #[serde(default = "default_templates_dir")]
    pub templates: PathBuf => {
        on_env -> ::std::env::var("EMAILS_TEMPLATE_DIR").map(|p| p.parse().expect("unable to parse into PathBuf")).unwrap_or(default_templates_dir());
        on_default -> default_templates_dir();
    },

    ///
    /// Logging configuration
    #[serde(default)]
    pub logging: LogConfig => {
        on_env -> LogConfig::from_env();
        on_default -> LogConfig::default();
    },

    ///
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig => {
        on_env -> ServerConfig::from_env();
        on_default -> ServerConfig::default();
    }
});

fn default_templates_dir() -> PathBuf {
    "./templates".parse().expect("unable to parse into PathBuf")
}

generate_config_struct!(SmtpConfig {
    ///
    /// The SMTP host to connect to
    #[serde(default = "localhost")]
    pub host: String => {
        on_env -> ::std::env::var("EMAILS_SMTP_HOST").unwrap_or(localhost());
        on_default -> localhost();
    },

    ///
    /// The SMTP port to connect to
    #[serde(default = "default_smtp_port")]
    pub port: u16 => {
        on_env -> ::std::env::var("EMAILS_SMTP_PORT").map(|p| p.parse::<u16>().expect("unable to parse value to u16")).unwrap_or(default_smtp_port());
        on_default -> default_smtp_port();
    }
});

fn localhost() -> String {
    "127.0.0.1".into()
}

fn default_smtp_port() -> u16 {
    587
}

generate_config_struct!(ServerConfig {
    ///
    /// Server port to bind to. By default, the email service will bind to
    /// `32121`.
    #[serde(default = "default_port")]
    pub port: u16 => {
        on_env -> ::std::env::var("EMAILS_SERVER_PORT").map(|p| p.parse::<u16>().expect("unable to parse value to u16")).unwrap_or(default_port());
        on_default -> default_port();
    },

    ///
    /// The host string to bind to. By default, the email service will bind to
    /// `0.0.0.0`.
    #[serde(default = "default_host")]
    pub host: String => {
        on_env -> ::std::env::var("EMAILS_SERVER_HOST").unwrap_or(default_host());
        on_default -> default_host();
    }
});

fn default_host() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    32121
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
        on_env -> ::std::env::var("EMAILS_LOG_IN_JSON").map(|f| f.parse::<bool>().expect("Unable to parse into boolean")).unwrap_or(false);
        on_default -> false;
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

impl Config {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let fd = File::open(path.as_ref()).map_err(Error::Io)?;
        serde_yaml::from_reader(fd).map_err(Error::YamlSerialization)
    }

    pub fn load<P: AsRef<Path>>(path: Option<P>) -> Result {
        if CONFIG.get().is_some() {
            warn!("Configuration file was already loaded!");
            return Ok(());
        }

        if path.is_none() {
            return Config::load(Some("./config.yml"));
        }

        let path = path.unwrap();
        let (path, path_string) = {
            let p = path.as_ref();
            (p, p.to_str().unwrap())
        };

        info!("attempting to load config in [{path_string}]");
        match Config::from_file(path) {
            Ok(config) => {
                info!("...successfully loaded from path [{path_string}]");
                CONFIG.set(config).unwrap();
                Ok(())
            }

            Err(e) => match e {
                Error::Io(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    CONFIG.set(Config::from_env()).unwrap();
                    Ok(())
                }

                Error::Io(err) => {
                    println!("[preinit::error] Unable to load configuration from path [{path_string}] due to IO error: {err} -- opting to use system environment variables instead!");
                    CONFIG.set(Config::from_env()).unwrap();
                    Ok(())
                }

                err => Err(err),
            },
        }
    }

    pub fn get<'a>() -> &'a Config {
        CONFIG.get().unwrap()
    }

    pub fn http_addr(&self) -> Result<SocketAddr> {
        let server = self.server.clone();
        format!("{}:{}", server.host, server.port)
            .parse()
            .map_err(|e| to_dyn_error!(e))
    }
}
