FROM rust:1-alpine AS builder

# musl-dev for the C toolchain rustls' crypto backend needs; curl for the
# Tailwind download below.
RUN apk add --no-cache musl-dev curl

# topcoat's build script only ever downloads the *glibc* Tailwind binary, which
# cannot run on Alpine. Install the musl build and point build.rs at it.
ARG TAILWIND_VERSION=4.3.2
RUN curl -fsSLo /usr/local/bin/tailwindcss \
        "https://github.com/tailwindlabs/tailwindcss/releases/download/v${TAILWIND_VERSION}/tailwindcss-linux-x64-musl" && \
    chmod +x /usr/local/bin/tailwindcss && \
    tailwindcss --help >/dev/null
ENV TAILWIND_CLI=/usr/local/bin/tailwindcss

# The bundler scans the compiled binary for asset declarations, so it ships as
# the `topcoat` CLI rather than a library call.
RUN cargo install topcoat-cli --locked

WORKDIR /work
COPY . .

# The asset bundle is per-profile (asset IDs embed $OUT_DIR), so it must be
# bundled with the same `--release` build that gets copied into the runtime.
RUN cargo build --release && \
    topcoat asset bundle --release

FROM alpine:3.21 AS runner

RUN apk add --no-cache libgcc

# topcoat reads both: HOST defaults to 127.0.0.1 and PORT to 3000, neither of
# which is reachable from outside the container.
ENV HOST="0.0.0.0"
ENV PORT="8080"

WORKDIR /app

COPY --from=builder /work/target/release/amackerel /app/
# `AssetBundle::load()` reads `assets/manifest.toml` in the executable's own
# directory and nowhere else, so the bundle sits next to the binary.
COPY --from=builder /work/target/release/assets /app/assets

EXPOSE 8080

ENTRYPOINT ["/app/amackerel"]
