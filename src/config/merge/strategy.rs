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

//! Common merge strategies for primitives. This is made since the default implementations
//! might not what you want, so this is some common ones that can be overwritten with the
//! `Merge` proc-macro, or written by hand without it.

/// Common merge strategies for strings. The default strategy will compare the strings
/// and checks if `lhs` != `rhs`. This comes with the `append` and `overwrite` strategies:
///
/// * `overwrite_empty` will overwrite `right` into `left` if `left` was empty.
/// * `overwrite` will overwrite `right` into `left` regardless
/// * `append` will append `right` into `left`.
///
/// For string slices (`&str`), it is impossible to do since string slices are immutable
/// while [`String`] is mutable, so we don't plan to add `&str` support without doing
/// unsafe code.
pub mod strings {
    /// Overwrites the left hand-side into the right-hand side if lhs was empty.
    ///
    /// ## Example
    /// ```no_run
    /// # use noelware_merge::strategy::strings::overwrite_empty;
    /// #
    /// let mut a = String::new();
    /// let b = String::from("overwritten!");
    ///
    /// overwrite_empty(&mut a, b);
    /// assert_eq!(a.as_str(), "overwritten!");
    /// ```
    pub fn overwrite_empty(left: &mut String, right: String) {
        if left.is_empty() {
            *left = right;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::strings;

    #[test]
    fn strings_overwrite_empty() {
        let mut a = String::new();
        strings::overwrite_empty(&mut a, String::from("weow"));

        assert_eq!("weow", a);
        strings::overwrite_empty(&mut a, String::from("heck"));

        assert_eq!("weow", a);
    }
}
