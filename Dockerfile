FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

COPY migrations ./migrations
COPY templates ./templates
COPY static ./static
COPY .env ./.env
COPY diesel.toml ./diesel.toml

RUN cargo build --release

# Use a base image with compatible glibc version for runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libsqlite3-0 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/dipak_site .

CMD ["./dipak_site"]
