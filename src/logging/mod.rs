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

use self::visitor::DefaultVisitor;
use chrono::Local;
use owo_colors::{colors::CustomColor, FgColorDisplay, OwoColorize, Stream};
use std::{io, io::Write as _, thread};
use tracing::{Level, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{registry::LookupSpan, Layer};

mod logstash;
mod visitor;

pub use logstash::TracingWriter;

pub struct LogLayer;

impl<S: Subscriber + for<'l> LookupSpan<'l>> Layer<S> for LogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.normalized_metadata();
        let metadata = metadata.as_ref().unwrap_or_else(|| event.metadata());

        // grab a lock for writing to stdout
        let mut writer = io::stdout().lock();
        let (b1, b2) = (
            "[".if_supports_color(Stream::Stdout, gray_fg),
            "]".if_supports_color(Stream::Stdout, gray_fg),
        );

        let time = Local::now().format("[%B %d, %G - %H:%M:%S %p]");

        write!(
            writer,
            "{} {} {b1}",
            time.if_supports_color(Stream::Stdout, |x| x.fg_rgb::<134, 134, 134>()),
            match *metadata.level() {
                Level::TRACE => "TRACE"
                    .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<163, 182, 138>())
                    .bold()
                    .to_string(),

                Level::DEBUG => "DEBUG"
                    .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<163, 182, 138>())
                    .bold()
                    .to_string(),

                Level::INFO => "INFO "
                    .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<178, 157, 243>())
                    .bold()
                    .to_string(),

                Level::WARN => "WARN "
                    .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<243, 243, 134>())
                    .bold()
                    .to_string(),

                Level::ERROR => "ERROR"
                    .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<153, 75, 104>())
                    .bold()
                    .to_string(),
            },
        )
        .unwrap();

        let target = format!(
            "{:>49} {:>25}",
            metadata.module_path().unwrap_or("unknown"),
            thread::current()
                .name()
                .unwrap_or("main")
                .if_supports_color(Stream::Stdout, |x| x.fg_rgb::<244, 181, 213>())
        );

        write!(
            writer,
            "{}",
            target.if_supports_color(Stream::Stdout, |x| x.fg_rgb::<120, 231, 255>())
        )
        .unwrap();

        write!(writer, "{b2} ~> ").unwrap();

        let mut visitor = DefaultVisitor::default();
        event.record(&mut visitor);

        let _ = writeln!(writer);
    }
}

fn gray_fg<'a>(x: &'a &'a str) -> FgColorDisplay<'a, CustomColor<134, 134, 134>, &str> {
    x.fg_rgb::<134, 134, 134>()
}
