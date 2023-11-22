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

use chrono::{DateTime, Utc};
use std::{env, error::Error, ffi::OsStr, path::PathBuf, process::Command, time::SystemTime};

fn execute<T>(command: T, args: &[&str]) -> Result<String, Box<dyn Error + 'static>>
where
    T: Into<String> + AsRef<OsStr>,
{
    let res = Command::new(command).args(args).output()?;
    Ok(String::from_utf8(res.stdout)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=protos/emails.proto");
    println!("cargo:rerun-if-changed=build.rs");

    let commit_hash = execute("git", &["rev-parse", "--short=8", "HEAD"]).unwrap_or_else(|_| "noeluwu8".into());
    let build_date = {
        let now = SystemTime::now();
        let utc: DateTime<Utc> = now.into();

        utc.to_rfc3339()
    };

    let rustc_version = rustc_version::version().map(|s| s.to_string()).unwrap();

    println!("cargo:rustc-env=SERVICE_RUSTC_VERSION={rustc_version}");
    println!("cargo:rustc-env=SERVICE_COMMIT_HASH={commit_hash}");
    println!("cargo:rustc-env=SERVICE_BUILD_DATE={build_date}");

    // build the protobufs
    tonic_build::configure()
        .file_descriptor_set_path(PathBuf::from(env::var("OUT_DIR").unwrap()).join("descriptor.bin"))
        .compile(&["./protos/emails.proto"], &["./protos"])
        .unwrap();

    Ok(())
}
