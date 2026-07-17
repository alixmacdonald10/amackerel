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
# The raw binary reads config from env (not Cargo.toml), so hashing must be
# switched on here; the manifest is read from ./hash.txt next to the binary.
ENV LEPTOS_HASH_FILES="true"

WORKDIR /app

COPY --from=builder /work/target/release/amackerel /app/
COPY --from=builder /work/target/site /app/site
# HashedStylesheet/HydrationScripts read the hash manifest next to the binary.
COPY --from=builder /work/target/release/hash.txt /app/
COPY --from=builder /work/posts /app/posts
COPY --from=builder /work/Cargo.toml /app/

EXPOSE 8080

ENTRYPOINT ["/app/amackerel"]
