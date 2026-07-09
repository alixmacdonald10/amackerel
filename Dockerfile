FROM rustlang/rust:nightly-alpine AS builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen
RUN npm install -g sass
RUN curl --proto '=https' --tlsv1.3 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM alpine:3.21 AS runner

RUN apk add --no-cache libgcc

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site

WORKDIR /app

COPY --from=builder /work/target/release/amackerel /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/posts /app/posts
COPY --from=builder /work/Cargo.toml /app/

EXPOSE 8080

ENTRYPOINT ["/app/amackerel"]
