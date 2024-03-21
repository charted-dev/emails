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

FROM rust:1.77-slim-bullseye AS build

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt upgrade -y && apt install -y curl libssl-dev libarchive-tools pkg-config protobuf-compiler libgit2

ENV CARGO_INCREMENTAL=1
WORKDIR /build

COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN DEBIAN_FRONTEND=noninteractive apt update && DEBIAN_FRONTEND=noninteractive apt install -y bash tini curl libssl-dev libgit2
WORKDIR /app/noelware/charted/emails

COPY --from=build /build/target/release/emails /app/noelware/charted/emails/bin/emails
COPY distribution/docker/scripts               /app/noelware/charted/emails/scripts
COPY distribution/docker/config                /app/noelware/charted/emails/config

# renovate: datasource=github-tags name=grpc-ecosystem/grpc-health-probe
ENV GRPC_HEALTH_PROBE_VERSION="v0.4.25"
RUN set -eux; \
    arch="$(uname -m)"; \
    case "${arch}" in \
        aarch64|arm64) \
            HEALTHPROBE_DOWNLOAD_URL="https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-arm64"; \
            ;; \
        amd64|x86_64) \
            HEALTHPROBE_DOWNLOAD_URL="https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64"; \
            ;; \
    esac; \
    curl -fsSL -o /usr/local/bin/grpc-healthprobe ${HEALTHPROBE_DOWNLOAD_URL}; \
    chmod +x /usr/local/bin/grpc-healthprobe;

RUN mkdir -p /var/lib/noelware/charted/emails
RUN groupadd -g 1001 noelware && \
  useradd -rm -s /bin/bash -g noelware -u 1001 noelware && \
  chown 1001:1001 /app/noelware/charted/emails && \
  chown 1001:1001 /var/lib/noelware/charted/emails && \
  chmod +x /app/noelware/charted/emails/scripts/docker-entrypoint.sh

ENV EMAILS_DISTRIBUTION_KIND=docker
EXPOSE 32121
VOLUME /var/lib/noelware/charted/emails

USER noelware
ENTRYPOINT ["/app/noelware/charted/emails/scripts/docker-entrypoint.sh"]
CMD ["/app/noelware/charted/emails/bin/emails"]
