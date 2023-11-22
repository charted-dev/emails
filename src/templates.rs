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

pub mod resolver;

use crate::{config::TryFromEnv, var};
use eyre::Report;
use remi_fs::FilesystemStorageConfig;
use serde::{Deserialize, Serialize};

/// Represents the configuration for how to resolve templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Config {
    /// Uses the local filesystem to find and use templates from. All files
    /// must be valid UTF-8 or the server will panic, but won't crash
    /// the whole program.
    Filesystem(FilesystemStorageConfig),

    /// Uses the Kubernetes API to resolve templates from a [`ConfigMap`](https://kubernetes.io/docs/concepts/configuration/configmap) reference.
    Kubernetes,

    /// Uses a Git repository to resolve templates from. It'll be mounted into `${templates.git.directory}/templates`. The
    /// resolver also supports SSH connections.
    Git,
}

impl Default for Config {
    fn default() -> Config {
        let config = FilesystemStorageConfig::new(String::from("./templates"));
        Config::Filesystem(config)
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Err = Report;

    fn try_from_env() -> Result<Self::Output, Self::Err> {
        match var!("EMAILS_TEMPLATE_RESOLVER", is_optional: true) {
            Some(resolver) => match resolver.as_str() {
                "filesystem" | "fs" => Ok(Default::default()),
                "kubernetes" => Ok(Config::Kubernetes),
                "git" => Ok(Config::Git),
                resolver => Err(eyre!(
                    "wanted [filesystem/fs, kubernetes, git]; received {resolver} instead"
                )),
            },
            None => Ok(Default::default()),
        }
    }
}
