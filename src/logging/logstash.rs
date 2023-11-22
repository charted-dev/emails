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

use super::visitor::JsonVisitor;
use crate::{COMMIT_HASH, VERSION};
use chrono::Local;
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Write, process, thread};
use tracing::{span, Event, Metadata, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{registry::LookupSpan, Layer};

struct JsonStorage(BTreeMap<String, Value>);

/// Represents a generic trait to implement for overridding the default [`on_event`][Layer::on_event]
/// behaviour for a [`TracingWriter`].
pub trait OnEventInvoker: Send + 'static {
    fn invoke(&self, event: &Event<'_>, metadata: &Metadata<'_>, spans: Vec<Value>) -> Value;
}

impl<F> OnEventInvoker for F
where
    F: Fn(&Event<'_>, &Metadata<'_>, Vec<Value>) -> Value + Send + 'static,
{
    fn invoke(&self, event: &Event<'_>, metadata: &Metadata<'_>, spans: Vec<Value>) -> Value {
        (self)(event, metadata, spans)
    }
}

/// Default implementation of a [`OnEventInvoker`].
fn default_event_invoker(event: &Event<'_>, metadata: &Metadata<'_>, spans: Vec<Value>) -> Value {
    let thread = thread::current();
    let now = Local::now();
    let pid = process::id();

    let mut tree = BTreeMap::new();
    let mut visitor = JsonVisitor(&mut tree);
    event.record(&mut visitor);

    let message = tree
        .remove("message")
        .unwrap_or(Value::String(String::from("{none provided}")));

    json!({
        "@timestamp": now.to_rfc3339(),
        "message": message,
        "labels": json!({
            "service": "charted-emails",
            "version": format!("{VERSION}+{COMMIT_HASH}"),
            "vendor": "Noelware, LLC. <team@noelware.org>"
        }),

        "metadata.module": metadata.module_path(),
        "metadata.file": metadata.file(),
        "metadata.line": metadata.line(),
        "thread.name": thread.name().unwrap_or("main"),
        "process.id": pid,
        "spans": spans,
        "fields": match tree.is_empty() {
            true => None,
            false => Some(tree),
        }
    })
}

/// Represents a generic [`Layer`] for implementing blocking access to any
/// type that implements `W`. This will emit JSON messages that follow
/// a similar structure to Logstash.
pub struct TracingWriter<W: Write> {
    _writer: W,
    event_invoker: Box<dyn OnEventInvoker>,
}

impl<W: Write + Send> From<W> for TracingWriter<W> {
    fn from(writer: W) -> Self {
        Self {
            _writer: writer,
            event_invoker: Box::new(default_event_invoker),
        }
    }
}

impl<W: Write + 'static, S: Subscriber + for<'l> LookupSpan<'l>> Layer<S> for TracingWriter<W> {
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut data = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut data);
        attrs.record(&mut visitor);
        span.extensions_mut().insert(JsonStorage(data));
    }

    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.span(span).unwrap();
        let mut exts = span.extensions_mut();
        let storage: &mut JsonStorage = exts.get_mut::<JsonStorage>().unwrap();

        let mut visitor = JsonVisitor(&mut storage.0);
        values.record(&mut visitor);
    }

    fn on_event(&self, event: &Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.normalized_metadata();
        let metadata = metadata.as_ref().unwrap_or_else(|| event.metadata());

        // first, we need to get all span metadata
        let mut spans = vec![];
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let ext = span.extensions();
                let storage = ext.get::<JsonStorage>().unwrap();
                let data = &storage.0;

                spans.push(json!({
                    // show `null` if there are no fields available
                    "fields": match data.is_empty() {
                        true => None,
                        false => Some(data)
                    },

                    "target": span.metadata().target(),
                    "level": metadata.level().as_str().to_lowercase(),
                    "name": span.metadata().name(),
                    "meta": json!({
                        "module": span.metadata().module_path(),
                        "file": span.metadata().file(),
                        "line": span.metadata().line(),
                    })
                }));
            }
        }

        let payload = self.event_invoker.invoke(event, metadata, spans);
        dbg!(payload);
        // let json = serde_json::to_vec(&payload).unwrap();

        // // this monstrority is the main reason why i hate my life
        // let mut_us = &mut *self;
        // let _ = mut_us.writer.write(&json);
    }
}
