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

RUN cargo install topcoat-cli@^0.6 --locked

WORKDIR /work
COPY . .

RUN cargo build --release && \
    topcoat asset bundle --release

FROM alpine:3.21 AS runner

RUN apk add --no-cache libgcc

RUN addgroup -S app && adduser -S app -G app

ENV HOST="0.0.0.0"
ENV PORT="8080"

WORKDIR /app

COPY --from=builder --chown=app:app /work/target/release/amackerel /app/
COPY --from=builder --chown=app:app /work/target/release/assets /app/assets

USER app

EXPOSE 8080

HEALTHCHECK --interval=5m --timeout=3s \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

ENTRYPOINT ["/app/amackerel"]
