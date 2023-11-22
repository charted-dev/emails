// 🐻‍❄️🌸 core-rs: Extra utilities that helps builds Noelware's Rust projects
// Copyright 2023 Noelware, LLC. <team@noelware.org>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module is made for [`core-rs`](https://github.com/Noelware/core-rs) ~ Noelware's
//! core Rust crates that are used in every Noelware project and can be only accessed via
//! Noelware's Cargo registry.
//!
//! As the registry, repository, and crates are not available to the public yet, this is
//! copied from what Noel has wrote for it, so it is fine if we copy it.
//!
//! The code for the strategy module is only what `charted-emails` will use in its [`Merge`]
//! implementations.

pub mod strategy;

/// Trait that allows you to merge together two objects into one easily. This
/// is mainly used for the [`noelware-config`] crate to allow merges between
/// defaults, system environment variables, file-loading, and command-line arguments.
///
/// This can be used to your advantage to allow deep merging.
///
/// ## Example
/// ```ignore
/// # use noelware_merge::Merge;
/// #
/// pub struct MyWrapper(u64);
/// #
/// impl Merge for MyWrapper {
///     fn merge(&mut self, other: Self) {
///         *self.0 = other.0;
///     }
/// }
/// ```
///
/// [`noelware-config`]: https://crates.noelware.cloud/-/noelware-config/docs/latest
pub trait Merge {
    /// Does the merging all-together by modifying `self` from `other`.
    fn merge(&mut self, other: Self);
}

impl Merge for String {
    fn merge(&mut self, other: Self) {
        if *self != other {
            *self = other;
        }
    }
}

impl<T> Merge for Option<T> {
    fn merge(&mut self, mut other: Self) {
        if !self.is_some() {
            *self = other.take();
        }
    }
}

impl Merge for u16 {
    fn merge(&mut self, other: Self) {
        // don't override if both are zero
        if *self == 0 && other == 0 {
            return;
        }

        // override if `other` is nonzero and self is zero
        if *self == 0 && other > 0 {
            *self = other;
            return;
        }

        // don't override if self is nonzero and other is zero
        if *self != 0 && other == 0 {
            return;
        }

        // fallback: comparison
        if *self != other {
            *self = other;
        }
    }
}

impl Merge for bool {
    fn merge(&mut self, other: Self) {
        if !*self && !other {
            return;
        }

        if *self && !other || !*self && other {
            *self = other;
        }
    }
}
