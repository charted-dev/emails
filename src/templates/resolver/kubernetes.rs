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
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    path::PathBuf,
};
use tracing::{info, instrument};

/// Represents a [`TemplateResolver`] implementation that uses a Kubernetes
/// [`ConfigMap`](https://kubernetes.io/docs/concepts/configuration/configmap) to resolve templates in a given namespace.
///
/// The resolver doesn't need to create or destroy ConfigMaps, all it needs is read permissions, which you can
/// easily configure with the [official Helm chart](https://charts.noelware.org/~/charted/emails).
///
/// > **WARNING**: The resolver supports using Kubernetes v1.26+ as of 15/10/23.
#[derive(Clone)]
pub struct KubernetesTemplateResolver {
    client: Client,
    namespace: Cow<'static, str>,
}

impl Debug for KubernetesTemplateResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("KubernetesTemplateResolver")
            .field("namespace", &self.namespace)
            .field("client", &"<kube::Client>")
            .finish()
    }
}

impl KubernetesTemplateResolver {
    /// Creates a new [`KubernetesTemplateResolver`] instance.
    pub async fn new(namespace: Cow<'static, str>) -> Result<KubernetesTemplateResolver> {
        info!(%namespace, "contacting Kubernetes API");

        Ok(KubernetesTemplateResolver {
            namespace,
            client: Client::try_default().await?,
        })
    }
}

#[async_trait]
impl TemplateResolver for KubernetesTemplateResolver {
    const NAME: &'static str = "kubernetes";

    #[instrument(
        name = "emails.resolvers.kubernetes.pull",
        skip_all,
        path = tracing::field::display(path.display()),
    )]
    async fn pull(&self, path: PathBuf) -> Result<Option<String>> {
        let path = path.strip_prefix("./").unwrap_or(&path);
        let Some(s) = path.to_str() else {
            return Err(eyre!("received invalid utf-8 path"));
        };

        let (name, filename) = match s.split_once('/') {
            Some((_, filename)) if filename.contains('/') => {
                return Err(eyre!("expected one slash (i.e, 'weow/fluff.tmpl')"))
            }

            Some(tuple) => tuple,
            None => return Err(eyre!("empty string or didn't find a '/' delimiter in path")),
        };

        if s.contains('/') {
            return Err(eyre!("path cannot contain more slashes"));
        }

        // find a config map
        let api = Api::<ConfigMap>::namespaced(self.client.clone(), &self.namespace);
        match api.get(name).await {
            Ok(cm) => {
                // first, we need to check if the filename that we were
                // given is a valid key in a ConfigMap
                for (at, ch) in filename.chars().enumerate() {
                    if ch.is_alphanumeric() {
                        continue;
                    }

                    if ch == '_' {
                        continue;
                    }

                    if ch == '-' {
                        continue;
                    }

                    if ch == '.' {
                        continue;
                    }

                    return Err(eyre!(
                        "filename given [{filename}@{at} (char '{ch}')] was not a valid ConfigMap key path"
                    ));
                }

                let Some(data) = cm.data else {
                    return Err(eyre!("expected `data` key in ConfigMap {name}"));
                };

                Ok(data.get(filename).cloned())
            }

            Err(e) => return Err(Report::from(e)),
        }
    }
}
