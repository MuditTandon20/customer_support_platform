FROM rust:1.90-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY openapi.json ./openapi.json
RUN cargo build --release --locked
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/* && useradd -m app
COPY --from=build /app/target/release/rust-api /usr/local/bin/rust-api
USER app
ENV API_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["rust-api"]
