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
    borrow::Cow,
    io::{Result as IoResult, Write},
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::Arc,
};

use fern::Dispatch;
use sentry::{
    integrations::{backtrace::AttachStacktraceIntegration, panic::PanicIntegration},
    types::Dsn,
};

use ansi_term::Colour::RGB;
use chrono::Local;
use log::Log;
use regex::Regex;
use sentry_log::{NoopLogger, SentryLogger};
use serde_json::json;

use crate::{
    error::{Error, Result},
    to_dyn_error, Config, COMMIT_HASH, VERSION,
};

lazy_static! {
    static ref ANSI_TERM_REGEX: Regex = Regex::new(r#"\u001b\[.*?m"#).unwrap();
}

/// Represents a writer that does nothing and discards information that is being written
/// into this writer.
#[derive(Debug, Default)]
pub struct NullWriter;

impl Write for NullWriter {
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        Ok(buf.len())
    }
}

unsafe impl Send for NullWriter {}

pub fn sentry(config: &Config) -> Result {
    if let Some(dsn) = &config.sentry_dsn {
        debug!("configuring sentry with DSN [{dsn}]!");
        let _ = sentry::init(sentry::ClientOptions {
            dsn: Some(Dsn::from_str(dsn.as_str()).map_err(|e| to_dyn_error!(e))?),
            release: Some(Cow::Owned(format!("{VERSION}+{COMMIT_HASH}"))),
            traces_sample_rate: 1.0,
            attach_stacktrace: true,
            integrations: vec![
                Arc::new(AttachStacktraceIntegration::default()),
                Arc::new(PanicIntegration::default()),
            ],

            ..Default::default()
        });
    }

    Ok(())
}

pub fn logging(config: &Config) -> Result {
    let logging = &config.logging;
    let level = logging.clone().level;
    let filter = level.to_level_filter();
    let log_in_json = logging.json;
    let console = Dispatch::new()
        .format(move |out, message, record| {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("main");
            let pid = std::process::id();
            let disable_colours = std::env::var("MAKUTU_DISABLE_COLOURS").is_ok();

            if disable_colours {
                out.finish(format_args!(
                    "{} {:<5} [{} <{}> ({})] :: {}",
                    Local::now().format("[%B %d, %G | %H:%M:%S %p]"),
                    record.level(),
                    record.target(),
                    pid,
                    name,
                    message
                ));
            } else if log_in_json {
                let level_name = record.level().as_str();
                let msg = format_args!("{message}").to_string();
                let data = json!({
                    "@timestamp": Local::now().to_rfc3339(),
                    "@version": "1",
                    "message": ANSI_TERM_REGEX.replace(msg.as_str(), ""),
                    "source": format!("Noelware/makutu v{VERSION}+{COMMIT_HASH}"),
                    "module": record.target(),
                    "thread": name,
                    "level": level_name,
                    "pid": pid,
                    "file": json!({
                        "path": record.file(),
                        "line": record.line()
                    })
                });

                out.finish(format_args!("{data}"));
            } else {
                let color = match record.level() {
                    log::Level::Error => RGB(153, 75, 104).bold(),
                    log::Level::Debug => RGB(163, 182, 138).bold(),
                    log::Level::Info => RGB(178, 157, 243).bold(),
                    log::Level::Trace => RGB(163, 182, 138).bold(),
                    log::Level::Warn => RGB(243, 243, 134).bold(),
                };

                let pid = std::process::id();
                let time = RGB(134, 134, 134).paint(format!(
                    "{}",
                    Local::now().format("[%B %d, %G | %H:%M:%S %p]")
                ));

                let level = color.paint(format!("{:<5}", record.level()));
                let (b1, b2) = (RGB(134, 134, 134).paint("["), RGB(134, 134, 134).paint("]"));
                let (p1, p2) = (RGB(134, 134, 134).paint("("), RGB(134, 134, 134).paint(")"));
                let target = RGB(120, 231, 255).paint(format!("{:<50}", record.target()));
                let thread_name = RGB(255, 105, 189).paint(format!("{name:>25}"));
                let pid_colour = RGB(169, 147, 227).paint(pid.to_string());

                out.finish(format_args!(
                   "{time} {level} {b1}{target} {thread_name} {p1}{pid_colour}{p2}{b2} :: {message}"
               ));
            }
        })
        .chain(std::io::stdout())
        .level(filter)
        .into_shared();

    let logstash = Dispatch::new()
        .format(move |out, message, record| {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("main");
            let pid = std::process::id();
            let level_name = record.level().as_str();
            let msg = format_args!("{message}").to_string();
            let data = json!({
                "@timestamp": Local::now().to_rfc3339(),
                "@version": "1",
                "message": ANSI_TERM_REGEX.replace(msg.as_str(), ""),
                "source": format!("Noelware/makutu v{VERSION}+{COMMIT_HASH}"),
                "module": record.target(),
                "thread": name,
                "level": level_name,
                "pid": pid,
                "file": json!({
                    "path": record.file(),
                    "line": record.line()
                })
            });

            out.finish(format_args!("{data}"));
        })
        .level(filter)
        .chain(if let Some(url) = logging.logstash_uri() {
            let addr = url
                .parse::<SocketAddr>()
                .map_err(|e| Error::ParseSocketError {
                    error: e,
                    value: url,
                })
                .expect("unable to parse to socket addr");

            Box::new(
                TcpStream::connect(addr)
                    .expect("Unable to connect to Logstash TCP stream!"),
            ) as Box<dyn Write + Send>
        } else {
            Box::<NullWriter>::default() as Box<dyn Write + Send>
        })
        .into_shared();

    let dispatch = Dispatch::new().chain(console).chain(logstash).chain(
        if config.sentry_dsn.is_some() {
            Box::new(SentryLogger::new())
        } else {
            Box::new(NoopLogger) as Box<dyn Log>
        },
    );

    dispatch.apply().map_err(|e| to_dyn_error!(e)).map(|_| ())
}
