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

use owo_colors::{OwoColorize, Stream};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fmt::Debug, io};
use tracing::field::{Field, Visit};

/// Represents a [`Visit`]or for recording tracing values into JSON-encodable values.
pub struct JsonVisitor<'a>(pub &'a mut BTreeMap<String, Value>);

macro_rules! impl_visitor_instructions {
    ($($name:ident => $ty:ty),*) => {
        $(
            fn $name(&mut self, field: &::tracing::field::Field, value: $ty) {
                self.0.insert(field.name().to_string(), ::serde_json::json!(value));
            }
        )*
    }
}

impl<'a> Visit for JsonVisitor<'a> {
    impl_visitor_instructions! {
        record_f64 => f64,
        record_i64 => i64,
        record_u64 => u64,
        record_i128 => i128,
        record_bool => bool,
        record_str => &str,
        record_u128 => u128
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.0.insert(field.name().to_string(), json!(format!("{value}")));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0.insert(field.name().to_string(), json!(format!("{value:?}")));
    }
}

pub struct DefaultVisitor {
    result: io::Result<()>,
    writer: Box<dyn io::Write + Send>,
}

impl Default for DefaultVisitor {
    fn default() -> Self {
        Self {
            result: Ok(()),
            writer: Box::new(io::stdout()),
        }
    }
}

impl Visit for DefaultVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if self.result.is_err() {
            return;
        }

        // don't emit messages from the `log` crate as those are translated
        // in the generic layer itself.
        if field.name().starts_with("log.") {
            return;
        }

        match field.name() {
            "message" => {
                self.result = write!(self.writer, "{value:?}");
            }

            field => {
                self.result = write!(
                    self.writer,
                    "{}",
                    format!(" {field}={value:?}")
                        .if_supports_color(Stream::Stdout, |txt| txt.fg_rgb::<134, 134, 134>())
                );
            }
        }
    }
}
