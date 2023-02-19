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

use tonic::{Request, Response, Status};

use crate::{
    protos::{
        emails_server::Emails, PingRequest, PingResponse, SendEmailRequest,
        SendEmailResponse,
    },
    Config, TemplateEngine,
};

#[derive(Debug, Clone)]
pub struct EmailService {
    _templates: TemplateEngine,
}

impl Default for EmailService {
    fn default() -> Self {
        let config = Config::get();
        Self {
            _templates: TemplateEngine::new(config.templates.clone()),
        }
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
        _request: Request<SendEmailRequest>,
    ) -> Result<Response<SendEmailResponse>, Status> {
        Ok(Response::new(SendEmailResponse {
            success: true,
            should_retry: false,
            error_message: None,
        }))
    }
}
