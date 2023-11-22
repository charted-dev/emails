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

use charted_emails::{
    config::{merge::Merge, Config, TryFromEnv},
    logging::LogLayer,
    service::Service,
    var, COMMIT_HASH, RUSTC_VERSION, VERSION,
};
use eyre::{eyre, Result};
use sentry_tracing::SentryLayer;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use tracing_subscriber::{prelude::*, registry};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = resolve_config().await?;
    let registry = registry()
        .with(config.sentry_dsn.as_ref().map(|_| SentryLayer::default()))
        .with(LogLayer);

    registry.try_init()?;

    info!(
        version = VERSION,
        commit.hash = COMMIT_HASH,
        rustc = RUSTC_VERSION,
        "starting `charted-emails` application..."
    );

    Service::new(config).await?.start().await
}
/// Finds a default configuration file that exists on disk.
fn find_default_location() -> Result<PathBuf> {
    let config_dir: PathBuf = PathBuf::from("./config");

    // 1. finds ./config/emails.yaml, which is true for the Docker/Helm deployment
    if config_dir.try_exists()? && config_dir.is_dir() {
        let file = config_dir.join("emails.yaml");
        if file.try_exists()? && file.is_file() {
            return Ok(file);
        }
    }

    // 2. finds ./config.yml, for most deployments so it is easier.
    let config_file = PathBuf::from("./config.yml");
    if config_file.try_exists()? && config_file.is_file() {
        return Ok(config_file);
    }

    Err(eyre!(
        "unable to find a configuration file in the following paths: \
        * ./config/emails.yaml \
        * ./config.yml"
    ))
}

async fn resolve_config() -> Result<Config> {
    let config_path = var!("EMAILS_CONFIG_FILE", to: PathBuf, or_else: find_default_location()?);
    let env = Config::try_from_env()?;

    let contents = fs::read_to_string(&config_path).await?;
    let serialized: Config = serde_yaml::from_str(&contents)?;

    let mut config = Config::default();
    config.merge(env);
    config.merge(serialized);

    Ok(config)
}
