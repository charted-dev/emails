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

FROM rust:1.68-alpine AS builder

RUN apk update && apk add --no-cache build-base openssl-dev gcompat libc6-compat && apk add --no-cache protobuf-dev --repository=https://dl-cdn.alpinelinux.org/alpine/edge/main

ENV CARGO_INCREMENTAL=1
WORKDIR /build

COPY Cargo.toml .
RUN echo "fn main() {}" >> dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
ENV RUSTFLAGS=-Ctarget-feature=-crt-static
RUN cargo build --release
RUN rm dummy.rs && sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY . .
RUN cargo build --release

FROM alpine:3.17

RUN apk update && apk add --no-cache bash tini openssl-dev gcompat libc6-compat libgcc openssl1.1-compat
WORKDIR /app/noelware/charted/emails

COPY --from=builder /build/target/release/emails /app/noelware/charted/emails/bin/emails
COPY distribution/docker/scripts                 /app/noelware/charted/emails/scripts

EXPOSE 32121
RUN addgroup -g 1001 noelware && \
    adduser -DSH -u 1001 -G noelware noelware && \
    chown 1001:1001 /app/noelware/charted/emails && \
    chmod +x /app/noelware/charted/emails/bin/emails /app/noelware/charted/emails/scripts/docker-entrypoint.sh

# Install gRPC Health Probe so you can health-check this application
ENV GRPC_HEALTH_PROBE_VERSION="0.4.15"
RUN set -eux; \
  arch="$(uname -m)"; \
  case "${arch}" in \
    aarch64|arm64) \
      HEALTHPROBE_DOWNLOAD_URL=" https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v${PROTOC_VERSION}/grpc_health_probe-linux-arm64"; \
      ;; \
    amd64|x86_64) \
      HEALTHPROBE_DOWNLOAD_URL=" https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v${PROTOC_VERSION}/grpc_health_probe-linux-amd64"; \
      ;; \
  esac; \
  mkdir -p /opt/healthprobe; \
  curl -fsSL -o /opt/healthprobe/healthprobe ${HEALTHPROBE_DOWNLOAD_URL} \
  chmod +x /opt/healthprobe/healthprobe;

RUN ln -s /opt/healthprobe/healthprobe /usr/bin/grpc-healthprobe

USER noelware
ENTRYPOINT ["/app/noelware/charted/emails/scripts/docker-entrypoint.sh"]
CMD ["/app/noelware/charted/emails/bin/emails"]
