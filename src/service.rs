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

use std::collections::HashMap;

use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Address,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use mustache::Data;
use prost_types::{value::Kind, ListValue};
use tonic::{Request, Response, Status};

use crate::{
    error::{Error, Result},
    protos::{
        emails_server::Emails, PingRequest, PingResponse, SendEmailRequest,
        SendEmailResponse,
    },
    Config, TemplateEngine, COMMIT_HASH, VERSION,
};

#[derive(Clone)]
pub struct EmailService {
    templates: TemplateEngine,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub async fn new() -> Result<EmailService> {
        info!("creating smtp mailer...");

        let config = Config::get();
        let mut mailer =
            AsyncSmtpTransport::<Tokio1Executor>::relay(config.smtp.host.as_str())
                .map_err(Error::SmtpError)?
                .port(config.smtp.port);

        match (config.smtp.username.clone(), config.smtp.password.clone()) {
            (Some(username), Some(password)) => {
                // so the credentials can be in the final transport
                // u dont know how much time it took to realise that i had
                // to do this
                //
                // end me
                mailer = mailer
                    .clone()
                    .credentials(Credentials::new(username, password));
            }

            (Some(_), None) => {
                return Err(Error::Message(
                    "Missing `smtp.password` configuration key".into(),
                ))
            }

            (None, Some(_)) => {
                return Err(Error::Message(
                    "Missing `smtp.username` configuration key".into(),
                ))
            }

            (None, None) => {}
        }

        let mailer = mailer.build();
        let success = mailer.test_connection().await?;
        if !success {
            warn!("unable to send NOOP request to email server, things might break D:");
        }

        let templates = TemplateEngine::new(config.templates.clone());
        templates.init()?;

        info!("created smtp mailer!");
        Ok(EmailService { templates, mailer })
    }
}

#[async_trait]
impl Emails for EmailService {
    async fn ping(
        &self,
        _request: Request<PingRequest>,
    ) -> tonic::Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse { pong: true }))
    }

    async fn send(
        &self,
        request: Request<SendEmailRequest>,
    ) -> Result<Response<SendEmailResponse>, Status> {
        let config = Config::get();
        let to = request.get_ref().to.clone();
        debug!("sending email to address [{to}]");

        let request = request.get_ref();
        if let Some(content) = &request.content {
            debug!("...with content\n{content}");

            let from_addr = match config.smtp.from.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    error!("Unable to parse from address [{}]: {e}", config.smtp.from);
                    sentry::capture_error(&e);

                    return Err(Status::internal("Internal Server Error"));
                }
            };

            let to_addr = match to.parse::<Address>() {
                Ok(addr) => addr,
                Err(e) => {
                    error!("Unable to parse to address [{}]: {e}", to);
                    sentry::capture_error(&e);

                    return Err(Status::internal("Internal Server Error"));
                }
            };

            let message = Message::builder()
                .from(Mailbox::new(None, from_addr))
                .to(Mailbox::new(None, to_addr))
                .subject(request.subject.clone())
                .date_now()
                .user_agent(format!("Noelware/charted-email (+https://github.com/charted-dev/email-service; v{VERSION}+{COMMIT_HASH}"))
                .body(content.clone())
                .map_err(|e| {
                    Status::internal(format!("Unable to create message payload: {e}"))
                })?;

            match self.mailer.send(message).await {
                Ok(_) => {
                    return Ok(Response::new(SendEmailResponse {
                        success: true,
                        should_retry: false,
                        error_message: None,
                    }));
                }

                Err(e) => {
                    error!("Unable to send email to [{to}]: {e}");
                    sentry::capture_error(&e);

                    return Ok(Response::new(SendEmailResponse {
                        success: false,
                        should_retry: false,
                        error_message: Some(format!(
                            "Unable to send email to [{to}]: {e}"
                        )),
                    }));
                }
            }
        }

        // First, we need to find the template that we need to
        // render.
        if request.template.is_none() {
            return Ok(Response::new(SendEmailResponse {
                success: false,
                should_retry: false,
                error_message: Some("Missing template to use".into()),
            }));
        }

        let template = request.template.as_ref().unwrap();
        debug!("...with template {template}!");
        match self.templates.find(template).await {
            Ok(true) => {}
            Ok(false) => {
                return Ok(Response::new(SendEmailResponse {
                    success: false,
                    should_retry: false,
                    error_message: Some(format!("Template {template} was not found.")),
                }))
            }

            Err(e) => {
                error!("Received i/o error when trying to find template {template}: {e}");
                sentry::capture_error(&e);

                return Err(Status::internal("Internal server error"));
            }
        }

        let context = match request.context.clone() {
            Some(s) => prost_value_to_data(Kind::StructValue(s)),
            None => Data::Map(HashMap::<String, Data>::new()),
        };

        let rendered = self
            .templates
            .render(template, context)
            .await
            .map_err(|e| {
                error!("Unable to render template {template}: {e}");
                sentry::capture_error(&e);

                Status::internal(format!("Unable to render template {template}: {e}"))
            })?;

        trace!("rendered result:\n{rendered}");
        let from_addr = match config.smtp.from.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Unable to parse from address [{}]: {e}", config.smtp.from);
                sentry::capture_error(&e);

                return Err(Status::internal("Internal Server Error"));
            }
        };

        let to_addr = match to.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Unable to parse to address [{}]: {e}", to);
                sentry::capture_error(&e);

                return Err(Status::internal("Internal Server Error"));
            }
        };

        let message = Message::builder()
                .from(Mailbox::new(None, from_addr))
                .to(Mailbox::new(None, to_addr))
                .subject(request.subject.clone())
                .date_now()
                .user_agent(format!("Noelware/charted-email (+https://github.com/charted-dev/email-service; v{VERSION}+{COMMIT_HASH}"))
                .body(rendered)
                .map_err(|e| {
                    Status::internal(format!("Unable to create message payload: {e}"))
                })?;

        match self.mailer.send(message).await {
            Ok(_) => Ok(Response::new(SendEmailResponse {
                success: true,
                should_retry: false,
                error_message: None,
            })),

            Err(e) => {
                error!("Unable to send email to [{to}]: {e}");
                sentry::capture_error(&e);

                Ok(Response::new(SendEmailResponse {
                    success: false,
                    should_retry: false,
                    error_message: Some(format!("Unable to send email to [{to}]: {e}")),
                }))
            }
        }
    }
}

fn prost_value_to_data(value: Kind) -> Data {
    match value {
        Kind::BoolValue(b) => Data::Bool(b),
        Kind::NullValue(_) => Data::Null,
        Kind::NumberValue(float) => Data::String(float.to_string()),
        Kind::StringValue(s) => Data::String(s),
        Kind::StructValue(s) => {
            let mut res = HashMap::new();
            for (key, value) in s.fields {
                if value.kind.is_none() {
                    warn!(
                        "key [{key}] with value kind {:?} couldn't be determined, skipping!", value.kind
                    );

                    continue;
                }

                res.insert(key, prost_value_to_data(value.kind.clone().unwrap()));
            }

            Data::Map(res)
        }

        Kind::ListValue(ListValue { values }) => {
            let mut res: Vec<Data> = vec![];
            for (index, val) in values.iter().enumerate() {
                if val.kind.is_none() {
                    warn!("value kind in index #{index} ({:?}) couldn't be determined, will be skipped", val.kind);
                    continue;
                }

                res.push(prost_value_to_data(val.kind.clone().unwrap()));
            }

            Data::Vec(res)
        }
    }
}
