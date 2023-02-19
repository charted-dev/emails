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
