#!/bin/bash

# 🐻‍❄️💌 email-service: charted's email service built in Rust that can be connected via gRPC
# Copyright 2023 Noelware, LLC. <team@noelware.org>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -o errexit
set -o nounset
set -o pipefail

. /app/noelware/charted/emails/scripts/liblog.sh

if ! [[ "${EMAILS_ENABLE_WELCOME_PROMPT:-yes}" =~ ^(no|false|0)$ ]]; then
  info ""
  info "  Welcome to the ${BOLD}charted's email microservice${RESET} container image."
  info "  🐻‍❄️💌 charted's email service built in Rust that can be connected via gRPC"
  info ""
  info "  * Subscribe to the project for updates:        https://github.com/charted-dev/email-service"
  info "  * Any issues occur? Report it to us at GitHub: https://github.com/charted-dev/email-service/issues"
  info ""
fi

debug "$ tini -s $@"
tini -s "$@"
