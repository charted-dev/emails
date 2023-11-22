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

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate eyre;

/// Constant that points to the current Rust compiler version.
pub const RUSTC_VERSION: &str = env!("SERVICE_RUSTC_VERSION");

/// Constant that points to the Git commit from the [canonical repository](https://github.com/charted-dev/emails-service).
pub const COMMIT_HASH: &str = env!("SERVICE_COMMIT_HASH");

/// Constant that points to a RFC3339-formatted timestamp of when the crate was last built.
pub const BUILD_DATE: &str = env!("SERVICE_BUILD_DATE");

/// Constant that points to the version of `emails-service` you're using.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;
pub mod logging;
pub mod service;
pub mod templates;

pub(crate) mod protos {
    tonic::include_proto!("noelware.charted.emails");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
}

pub use protos::{
    emails_server::{Emails, EmailsServer},
    Error, PingRequest, PingResponse, SendEmailRequest, SendEmailResponse,
};
