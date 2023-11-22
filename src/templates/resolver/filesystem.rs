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

use std::path::PathBuf;

use super::TemplateResolver;
use eyre::{Report, Result};
use remi_core::StorageService;
use remi_fs::{FilesystemStorageConfig, FilesystemStorageService};

/// Represents a [`TemplateResolver`] that uses the local filesystem as the
/// resolver's root directory.
#[derive(Debug, Clone)]
pub struct FilesystemTemplateResolver(FilesystemStorageService);

impl FilesystemTemplateResolver {
    pub fn new(config: FilesystemStorageConfig) -> FilesystemTemplateResolver {
        FilesystemTemplateResolver(FilesystemStorageService::with_config(config))
    }
}

#[async_trait]
impl TemplateResolver for FilesystemTemplateResolver {
    async fn init(&self) -> Result<()> {
        self.0.init().await.map_err(Report::from)
    }

    async fn pull(&self, path: PathBuf) -> Result<Option<String>> {
        self.0
            .open(path)
            .await
            .map(|bytes| bytes.map(|b| String::from_utf8(b.to_vec()).expect("expected valid utf-8 encoded text")))
            .map_err(Report::from)
    }
}
