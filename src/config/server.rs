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

use super::{
    merge::{self, Merge},
    TryFromEnv,
};
use crate::var;
use eyre::Report;
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::PathBuf};

/// Configuration for the gRPC server itself.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    /// Server host to bind to, use `127.0.0.1` for local connections or the default,
    /// `0.0.0.0` to listen on all network interfaces.
    #[serde(default = "host")]
    pub host: String,

    /// 16-bit port to use when binding to a TCP socket; default is always `32121`.
    #[serde(default = "port")]
    pub port: u16,

    /// Configures the TLS configuration when creating the gRPC server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

impl TryFromEnv for Config {
    type Output = Config;
    type Err = Report;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        Ok(Config {
            host: var!("EMAILS_SERVER_HOST", or_else: var!("HOST", or_else: host())),
            port: var!("EMAILS_SERVER_PORT", to: u16, or_else: var!("PORT", to: u16, or_else: port())),
            tls: match var!("EMAILS_SERVER_TLS_CA_ROOT", is_optional: true) {
                Some(_) => Some(TlsConfig::try_from_env()?),
                None => None,
            },
        })
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        merge::strategy::strings::overwrite_empty(&mut self.host, other.host);
        self.port.merge(other.port);
        self.tls.merge(other.tls);
    }
}

impl Config {
    pub fn addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("unable to construct SocketAddr from configured 'config.server.host':'config.server.port'")
    }
}

#[inline(always)]
fn host() -> String {
    String::from("0.0.0.0")
}

#[inline(always)]
const fn port() -> u16 {
    32121
}

/// Represents the configuration for configuring TLS for the gRPC service.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TlsConfig {
    /// String-encoded CA certificate or a path to a certificate on the filesystem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_root: Option<String>,

    /// Whether or not if client authentication is optional when allowing
    /// clients to flow through. Default is `false` and is not recommended
    /// to be set to `true` unless debugging TLS issues.
    #[serde(default)]
    pub auth_optional: bool,
}

impl TryFromEnv for TlsConfig {
    type Output = TlsConfig;
    type Err = Report;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        Ok(TlsConfig {
            ca_root: var!("EMAILS_SERVER_TLS_CA_ROOT", is_optional: true)
                .map(|d| find_ca_root(d).expect("unable to resolve path or certificate data")),

            auth_optional: var!("EMAILS_SERVER_TLS_AUTH_OPTIONAL", to: bool, or_else: false),
        })
    }
}

/// Finds a certificate and returns a string of the encoded PEM data of a certificate.
fn find_ca_root(path_or_ct: String) -> eyre::Result<String> {
    let path = PathBuf::from(&path_or_ct);
    match path.components().next() {
        Some(_) => fs::read_to_string(path).map_err(Report::from),
        None => Ok(path_or_ct),
    }
}

impl Merge for TlsConfig {
    fn merge(&mut self, other: Self) {
        self.auth_optional.merge(other.auth_optional);
        self.ca_root.merge(other.ca_root);
    }
}
