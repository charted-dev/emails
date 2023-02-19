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

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT_HASH: &str = env!("SERVICE_COMMIT_HASH");
pub const BUILD_DATE: &str = env!("SERVICE_BUILD_DATE");

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate async_trait;

pub mod error;
pub mod setup_utils;

mod config;
mod service;
mod template_engine;

pub use config::*;
pub use service::*;
pub use template_engine::*;

mod protos {
    tonic::include_proto!("noelware.charted.emails");
}

pub use protos::*;
