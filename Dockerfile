# syntax=docker/dockerfile:1@sha256:87999aa3d42bdc6bea60565083ee17e86d1f3339802f543c0d03998580f9cb89

FROM --platform=$BUILDPLATFORM ghcr.io/usa-reddragon/rust-cross:1.94.1@sha256:57b5216b0628d708d3e629fce9014ad3c1c25aacb5f3984b924ebacb0dd0bba9 AS builder

ARG TARGETARCH
ARG PKG_VERSION=dev
ARG GIT_COMMIT=unknown

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Map Docker platform to Rust target triple and musl cross-compiler
RUN case "${TARGETARCH}" in \
      amd64) \
        echo "x86_64-unknown-linux-musl" > /tmp/rust_target && \
        echo "x86_64-linux-musl-gcc" > /tmp/cc ;; \
      arm64) \
        echo "aarch64-unknown-linux-musl" > /tmp/rust_target && \
        echo "aarch64-linux-musl-gcc" > /tmp/cc ;; \
      *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac

WORKDIR /build
COPY . .

RUN export RUST_TARGET="$(cat /tmp/rust_target)" && \
    export CROSS_CC="$(cat /tmp/cc)" && \
    export CC_$(echo "${RUST_TARGET}" | tr '-' '_')="${CROSS_CC}" && \
    export CARGO_TARGET_$(echo "${RUST_TARGET}" | tr '-' '_' | tr '[:lower:]' '[:upper:]')_LINKER="${CROSS_CC}" && \
    export PKG_VERSION="${PKG_VERSION}" && \
    export GIT_COMMIT="${GIT_COMMIT}" && \
    cargo build --release --target "${RUST_TARGET}" && \
    cp "target/${RUST_TARGET}/release/mqtt-wx" /mqtt-wx

FROM scratch

COPY --from=alpine:latest@sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=builder /mqtt-wx /mqtt-wx

USER 65534:65534

ENTRYPOINT ["/mqtt-wx"]
