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

use std::{
    fs::create_dir,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};
use tokio::{fs::File, io::AsyncReadExt};

/// Represents the engine for converting templates in a directory into what the service
/// can send. This is an abstraction over Mustache templates.
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    templates_dir: PathBuf,
}

impl TemplateEngine {
    /// Creates a new [`TemplateEngine`] with the directory of the template.
    pub fn new<P: AsRef<Path>>(path: P) -> TemplateEngine {
        TemplateEngine {
            templates_dir: path.as_ref().to_path_buf(),
        }
    }

    pub fn init(&self) -> Result {
        info!("initializing template engine...");

        if !self.templates_dir.exists() {
            warn!(
                "templates in directory [{:?}] doesn't exist",
                self.templates_dir
            );

            create_dir(self.templates_dir.clone()).map_err(Error::Io)?;
        }

        Ok(())
    }

    /// Could the file in the path be found?
    pub async fn find<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match File::open(self.templates_dir.join(path.as_ref())).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Renders a template with the given path and context object
    pub async fn render<P: AsRef<Path>, C: Into<mustache::Data>>(
        &self,
        path: P,
        context: C,
    ) -> Result<String> {
        let template = self.templates_dir.join(path.as_ref());
        if !template.exists() {
            return Err(Error::TemplateNotFound { template });
        }

        debug!("found template [{:?}]", template.display());
        let mut file = File::open(template).await?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;

        // Now, let's put it into Mustache
        let compiled_template = mustache::compile_str(buf.as_str())?;
        Ok(compiled_template.render_data_to_string(&context.into())?)
    }
}
