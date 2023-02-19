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

use std::env::var;

use emails::{
    emails_server::EmailsServer, error::Result, setup_utils, to_dyn_error, Config,
    EmailService, COMMIT_HASH, VERSION,
};
use log::*;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result {
    dotenv::dotenv().unwrap_or_default();
    match var("EMAILS_CONFIG_FILE") {
        Ok(p) => Config::load(Some(p))?,
        Err(_) => Config::load::<String>(None)?,
    };

    let config = Config::get();
    setup_utils::logging(config)?;
    setup_utils::sentry(config)?;

    info!("email service v{}+{} - initializing", VERSION, COMMIT_HASH);
    let http_addr = config.http_addr()?;

    info!("listening on http addr {http_addr}!");
    Server::builder()
        .add_service(EmailsServer::new(EmailService::default()))
        .serve(http_addr)
        .await
        .map_err(|e| to_dyn_error!(e))
}
