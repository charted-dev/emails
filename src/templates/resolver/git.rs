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

use super::TemplateResolver;
use eyre::{Report, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, instrument, warn};

/// Represents a [`TemplateResolver`] that uses Git repositories to locate
/// template files.
#[derive(Debug, Clone)]
pub struct GitTemplateResolver {
    // the root path is the canonical path to use when fetching templates
    root_path: PathBuf,
}

impl GitTemplateResolver {
    pub fn new<P: AsRef<Path>>(root: P) -> GitTemplateResolver {
        GitTemplateResolver {
            root_path: root.as_ref().into(),
        }
    }
}

#[async_trait]
impl TemplateResolver for GitTemplateResolver {
    const NAME: &'static str = "git";

    async fn init(&self) -> Result<()> {
        if !self.root_path.try_exists()? {
            warn!(
                root = tracing::field::display(self.root_path.display()),
                "root path for Git template resolver doesn't exist, creating"
            );

            fs::create_dir_all(self.root_path.clone()).await?;
        }

        Ok(())
    }

    #[instrument(
        name = "emails.resolvers.git.pull",
        skip_all,
        root = tracing::field::display(self.root_path.display()),
        path = tracing::field::display(path.display()),
    )]
    async fn pull(&self, path: PathBuf) -> Result<Option<String>> {
        info!("pulling path from root directory");

        let canon = path.canonicalize()?;
        if !canon.starts_with(self.root_path.clone()) {
            return Err(eyre!(
                "path {} is outside of root path [{}]",
                canon.display(),
                self.root_path.display()
            ));
        }

        if !canon.try_exists()? {
            return Ok(None);
        }

        fs::read_to_string(canon).await.map(Some).map_err(Report::from)
    }
}
