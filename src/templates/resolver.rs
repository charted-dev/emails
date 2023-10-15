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

use eyre::Result;
use std::path::PathBuf;

pub mod filesystem;
pub mod git;
pub mod kubernetes;

/// Represents a trait that allows to resolve templates from any canonical source.
#[async_trait]
pub trait TemplateResolver {
    /// Returns the name of this [`TemplateResolver`].
    const NAME: &'static str;

    /// Allows this [`TemplateResolver`] to do pre-initialization (i.e, pull git repositories or load
    /// the Kubernetes configuration).
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    /// Pulls a `path` from a specific source and returns the pulled contents
    /// of the template that we want to use.
    ///
    /// For example, if we need a `./weow/fluff.tmpl` from a Git source (depending
    /// on if [`init`][TemplateResolver::init] was called) will point to the git-pulled
    /// repository and point it to there.
    ///
    /// For the Kubernetes resolver, slashes are not allowed expect in first 2 characters,
    /// which will be stripped if found.
    async fn pull(&self, path: PathBuf) -> Result<Option<String>>;
}
