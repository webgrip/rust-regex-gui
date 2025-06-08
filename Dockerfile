FROM rust:1.87.0-slim-bookworm AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libgtk-3-0 \
        libxkbcommon0 \
        libxkbcommon-x11-0 \
        libx11-xcb1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/rust-regex-gui /usr/local/bin/regex-gui
CMD ["regex-gui"]
