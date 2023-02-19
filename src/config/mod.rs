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
mod server;
mod smtp;

use crate::{error::*, to_dyn_error};
use std::{
    fs::File,
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub use logging::*;
use once_cell::sync::OnceCell;
pub use server::*;
pub use smtp::*;

static CONFIG: OnceCell<Config> = OnceCell::new();

pub trait FromEnv<T> {
    fn from_env() -> T;
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

        impl $crate::FromEnv<$name> for $name {
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
    },

    ///
    /// SMTP configuration
    #[serde(default)]
    pub smtp: SmtpConfig => {
        on_env -> SmtpConfig::from_env();
        on_default -> SmtpConfig::default();
    }
});

fn default_templates_dir() -> PathBuf {
    "./templates".parse().expect("unable to parse into PathBuf")
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
