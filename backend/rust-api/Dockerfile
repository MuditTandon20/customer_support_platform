FROM rust:1.97-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY openapi.json ./openapi.json
RUN cargo test --release --locked && cargo build --release --locked
FROM debian:trixie-slim
RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends ca-certificates libssl3t64 && rm -rf /var/lib/apt/lists/* && useradd -m app
RUN find / -xdev -type f -perm /6000 -exec chmod a-s {} +
COPY --from=build /app/target/release/rust-api /usr/local/bin/rust-api
USER app
ENV API_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["rust-api"]
