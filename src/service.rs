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

use crate::{
    config::Config,
    protos,
    templates::{
        self,
        resolver::{filesystem::FilesystemTemplateResolver, TemplateResolver},
    },
    Emails, EmailsServer, Error, PingRequest, PingResponse, SendEmailRequest, SendEmailResponse, COMMIT_HASH, VERSION,
};
use eyre::{Context, Result};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Address, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor,
};
use mustache::Data;
use prost_types::{value::Kind, ListValue};
use sentry::{types::Dsn, ClientInitGuard};
use sentry_tower::NewSentryLayer;
use std::{borrow::Cow, collections::HashMap, path::PathBuf, str::FromStr};
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::health_reporter;
use tracing::{debug, error, info, trace, warn};

/// Represents an implementation of the `charted-emails` gRPC server.
pub struct Service {
    _sentry_guard: Option<ClientInitGuard>,
    resolver: Box<dyn TemplateResolver>,
    config: Config,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl Service {
    /// Creates a new [`Service`] instance.
    pub async fn new(config: Config) -> Result<Service> {
        let resolver: Box<dyn TemplateResolver> = match config.templates {
            templates::Config::Filesystem(ref cfg) => Box::new(FilesystemTemplateResolver::new(cfg.clone())),
            _ => unimplemented!(),
        };

        resolver.init().await?;

        let mut mailer = match config.smtp.starttls {
            true => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)?,
            false => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)?,
        }
        .port(config.smtp.port);

        match (config.smtp.username.clone(), config.smtp.password.clone()) {
            (Some(username), Some(password)) => {
                mailer = mailer.clone().credentials(Credentials::new(username, password));
            }

            (Some(_), None) => {
                return Err(eyre!(
                    "unable to create smtp transport: missing 'config.smtp.password' variable"
                ));
            }

            (None, Some(_)) => {
                return Err(eyre!(
                    "unable to create smtp transport: missing 'config.smtp.username' variable"
                ));
            }

            // skip if we don't have a user or pass
            (None, None) => {}
        }

        let mailer = mailer.build();
        if !mailer.test_connection().await? {
            warn!("unable to send NOOP request to SMTP server, things might be shaky!");
        }

        Ok(Service {
            _sentry_guard: config.sentry_dsn.as_ref().map(|dsn| {
                sentry::init(sentry::ClientOptions {
                    dsn: Some(Dsn::from_str(dsn).expect("unable to parse Sentry DSN from 'config.sentry_dsn'")),
                    accept_invalid_certs: false,
                    attach_stacktrace: true,
                    auto_session_tracking: false,
                    release: Some(Cow::Owned(format!("v{VERSION}+{COMMIT_HASH}"))),
                    traces_sample_rate: 1.0,
                    ..Default::default()
                })
            }),
            resolver,
            config,
            mailer,
        })
    }

    /// Starts the gRPC server until a cancellation (CTRL+C) occurs or when
    /// the server unexpectely receives a shutdown signal.
    pub async fn start(self) -> Result<()> {
        let (mut reporter, service) = health_reporter();
        info!("successfully created the healthcheck reporter");
        reporter.set_serving::<EmailsServer<Service>>().await;

        info!("creating reflection server");
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(protos::FILE_DESCRIPTOR_SET)
            .build()?;

        let addr = self.config.server.addr();
        info!(%addr, "now listening on");

        Server::builder()
            .layer(NewSentryLayer::new_from_top())
            .add_service(service)
            .add_service(reflection)
            .add_service(EmailsServer::new(self))
            .serve(addr)
            .await
            .context("unable to run gRPC service")
    }
}

#[async_trait]
impl Emails for Service {
    async fn ping(&self, _: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse { pong: true }))
    }

    async fn send(&self, request: Request<SendEmailRequest>) -> Result<Response<SendEmailResponse>, Status> {
        let request = request.get_ref();
        debug!(to = request.to, "sending email to address");

        let from = self.config.smtp.from_addr.parse::<Address>().map_err(|e| {
            error!(addr = self.config.smtp.from_addr, error = %e, "unable to parse from address");
            sentry::capture_error(&e);

            Status::internal("Internal Server Error")
        })?;

        let to = request.to.parse::<Address>().map_err(|e| {
            error!(addr = request.to, error = %e, "unable to parse the 'request.to' address");
            sentry::capture_error(&e);

            Status::internal("Internal Server Error")
        })?;

        if let Some(content) = request.content.clone() {
            trace!(to = request.to, from = self.config.smtp.from_addr, "{content}");

            let message = Message::builder()
                .from(Mailbox::new(None, from.clone()))
                .to(Mailbox::new(None, to.clone()))
                .subject(&request.subject)
                .date_now()
                .user_agent(format!(
                    "Noelware/charted-emails (+https://github.com/charted-dev/emails; v{VERSION}+{COMMIT_HASH}"
                ))
                .body(content)
                .map_err(|e| {
                    error!(?to, ?from, error = %e, "unable to create message");
                    sentry::capture_error(&e);

                    Status::internal(e.to_string())
                })?;

            match self.mailer.send(message).await {
                Ok(_) => {
                    return Ok(Response::new(SendEmailResponse {
                        success: true,
                        errors: vec![],
                    }))
                }

                Err(e) => {
                    return Ok(Response::new(SendEmailResponse {
                        success: false,
                        errors: vec![Error {
                            code: String::from("UNABLE_TO_SEND_EMAIL"),
                            message: e.to_string(),
                            details: None,
                        }],
                    }))
                }
            }
        }

        let Some(ref template) = request.template else {
            return Ok(Response::new(SendEmailResponse {
                success: false,
                errors: vec![Error {
                    code: String::from("MISSING_ARG"),
                    message: String::from("missing 'request.template' argument"),
                    details: None,
                }],
            }));
        };

        debug!(?to, ?from, %template, "using template code");
        let Some(contents) = self.resolver.pull(PathBuf::from(template)).await.map_err(|e| {
            error!(%template, error = %e, "unable to pull template");
            sentry::capture_error(&*e);

            Status::internal("Internal Server Error")
        })?
        else {
            warn!(%template, "unknown template");
            return Err(Status::invalid_argument(format!("unknown template '{template}'")));
        };

        let context = request
            .context
            .as_ref()
            .map(|data| prost_value_to_data_type(Kind::StructValue(data.clone())))
            .unwrap_or(Data::Map(HashMap::default()));

        let compiled = mustache::compile_str(contents.as_str()).map_err(|e| {
            error!(%template, error = %e, "unable to compile mustache template");
            sentry::capture_error(&e);

            Status::internal("unable to compile mustache template ({template})")
        })?;

        let message = Message::builder()
            .from(Mailbox::new(None, from.clone()))
            .to(Mailbox::new(None, to.clone()))
            .subject(&request.subject)
            .date_now()
            .user_agent(format!(
                "Noelware/charted-emails (+https://github.com/charted-dev/emails; v{VERSION}+{COMMIT_HASH}"
            ))
            .body(compiled.render_data_to_string(&context).map_err(|e| {
                error!(%template, error = %e, "unable to compile mustache template");
                sentry::capture_error(&e);

                Status::internal("unable to compile mustache template ({template})")
            })?)
            .map_err(|e| {
                error!(?to, ?from, error = %e, "unable to create message");
                sentry::capture_error(&e);

                Status::internal(e.to_string())
            })?;

        match self.mailer.send(message).await {
            Ok(_) => {
                return Ok(Response::new(SendEmailResponse {
                    success: true,
                    errors: vec![],
                }))
            }

            Err(e) => {
                return Ok(Response::new(SendEmailResponse {
                    success: false,
                    errors: vec![Error {
                        code: String::from("UNABLE_TO_SEND_EMAIL"),
                        message: e.to_string(),
                        details: None,
                    }],
                }))
            }
        }
    }
}

fn prost_value_to_data_type(value: Kind) -> Data {
    match value {
        Kind::StringValue(s) => Data::String(s),
        Kind::NumberValue(num) => Data::String(num.to_string()),
        Kind::BoolValue(b) => Data::Bool(b),
        Kind::NullValue(_) => Data::Null,
        Kind::StructValue(s) => {
            let mut res = HashMap::new();
            for (key, value) in s.fields {
                if value.kind.is_none() {
                    warn!(kind = ?value.kind, %key, "cannot determine Mustache type for key with Protobuf type; skipping");
                    continue;
                }

                res.insert(key, prost_value_to_data_type(value.kind.unwrap()));
            }

            Data::Map(res)
        }

        Kind::ListValue(ListValue { values }) => Data::Vec(
            values
                .iter()
                .enumerate()
                .filter_map(|(idx, val)| match val.kind.clone() {
                    Some(kind) => Some(prost_value_to_data_type(kind)),
                    None => {
                        warn!(
                            index = idx,
                            "cannot determine Mustache type for index in Protobuf list; skipping"
                        );

                        None
                    }
                })
                .collect(),
        ),
    }
}
