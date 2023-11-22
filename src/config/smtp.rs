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

use super::{merge::Merge, FromEnv};
use crate::var;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Username for authenticating with the SMTP server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for authenticating with the SMTP server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// From address when sending the email from.
    #[serde(default = "default_smtp_from", rename = "from")]
    pub from_addr: String,

    /// SMTP host address to connect to when sending out emails.
    #[serde(default = "localhost")]
    pub host: String,

    /// 16-bit port address to connect to.
    #[serde(default = "default_smtp_port")]
    pub port: u16,

    /// If the `STARTTLS` option should be enabled when establishing a connection towards
    /// the SMTP server.
    #[serde(default)]
    pub starttls: bool,

    /// Whether or not if a SSL connection should be established when connecting.
    #[serde(default)]
    pub ssl: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            from_addr: default_smtp_from(),
            username: None,
            password: None,
            starttls: false,
            host: localhost(),
            port: default_smtp_port(),
            ssl: false,
        }
    }
}

impl FromEnv for Config {
    type Output = Config;

    fn from_env() -> Self::Output {
        Config {
            from_addr: var!("EMAILS_SMTP_FROM_ADDRESS", or_else: default_smtp_from()),
            username: var!("EMAILS_SMTP_USERNAME", is_optional: true),
            password: var!("EMAILS_SMTP_PASSWORD", is_optional: true),
            starttls: var!("EMAILS_SMTP_STARTTLS", to: bool, or_else: false),
            host: var!("EMAILS_SMTP_HOST", or_else: localhost()),
            port: var!("EMAILS_SMTP_PORT", to: u16, or_else: default_smtp_port()),
            ssl: var!("EMAILS_SMTP_SSL", to: bool, or_else: false),
        }
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        self.from_addr.merge(other.from_addr);
        self.username.merge(other.username);
        self.password.merge(other.password);
        self.starttls.merge(other.starttls);
        self.host.merge(other.host);
        self.port.merge(other.port);
        self.ssl.merge(other.ssl);
    }
}

#[inline(always)]
fn localhost() -> String {
    "127.0.0.1".into()
}

#[inline(always)]
const fn default_smtp_port() -> u16 {
    587
}

#[inline(always)]
fn default_smtp_from() -> String {
    "from@example.com".into()
}
