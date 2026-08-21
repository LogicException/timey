FROM --platform=$BUILDPLATFORM rust:1.87-bookworm AS builder
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        gcc-x86-64-linux-gnu \
        libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-gnu
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
ENV CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/timey-api /usr/local/bin/timey-api
ENV BIND=0.0.0.0:3000
EXPOSE 3000
CMD ["timey-api"]
