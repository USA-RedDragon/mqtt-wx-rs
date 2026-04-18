# syntax=docker/dockerfile:1@sha256:2780b5c3bab67f1f76c781860de469442999ed1a0d7992a5efdf2cffc0e3d769

FROM --platform=$BUILDPLATFORM ghcr.io/usa-reddragon/rust-cross:1.94.1@sha256:824108f3a93447345db239f23ae306d2d14ccf82430ad8de7f680ea3a4f9ce6e AS builder

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

COPY --from=alpine:latest@sha256:5b10f432ef3da1b8d4c7eb6c487f2f5a8f096bc91145e68878dd4a5019afde11 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=builder /mqtt-wx /mqtt-wx

USER 65534:65534

ENTRYPOINT ["/mqtt-wx"]
