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

use crate::generate_config_struct;

generate_config_struct!(SmtpConfig {
    ///
    /// Username for authenticating with the SMTP server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String> => {
        on_env -> ::std::env::var("EMAILS_SMTP_USERNAME").ok();
        on_default -> None;
    },

    ///
    /// Password for authenticating with the SMTP server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String> => {
        on_env -> ::std::env::var("EMAILS_SMTP_PASSWORD").ok();
        on_default -> None;
    },

    ///
    /// The from address when sending out the email to
    #[serde(default = "default_smtp_from")]
    pub from: String => {
        on_env -> ::std::env::var("EMAILS_SMTP_FROM").unwrap_or(default_smtp_from());
        on_default -> default_smtp_from();
    },

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
    },

    ///
    /// If starttls should be enabled when establishing a connection
    #[serde(default = "false_variant")]
    pub tls: bool => {
        on_env -> ::std::env::var("EMAILS_SMTP_STARTTLS").map(|f| f.parse::<bool>().expect("Unable to parse into boolean")).unwrap_or(false_variant());
        on_default -> false_variant();
    },

    ///
    /// If SSL should be enabled when establishing a connection
    #[serde(default = "false_variant")]
    pub ssl: bool => {
        on_env -> ::std::env::var("EMAILS_SMTP_SSL").map(|f| f.parse::<bool>().expect("Unable to parse into boolean")).unwrap_or(false_variant());
        on_default -> false_variant();
    }
});

fn localhost() -> String {
    "127.0.0.1".into()
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_from() -> String {
    "from@example.com".into()
}

fn false_variant() -> bool {
    false
}
