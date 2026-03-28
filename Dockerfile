# Stage 1: Build Phase
FROM rust:latest as builder
WORKDIR /usr/src/app

# Copy dependency manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY media ./media
COPY migrations ./migrations

# Build the release binary.
RUN cargo build --release

# Stage 2: Minimal Runtime Phase
FROM debian:bookworm-slim
WORKDIR /usr/local/bin

# Install runtime dependencies (PostgreSQL client lib and CA certificates)
RUN apt-get update && \
    apt-get install -y libpq-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment
COPY --from=builder /usr/src/app/target/release/dipak_site .

# Copy static assets and necessary rendering templates
COPY --from=builder /usr/src/app/static ./static
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/media ./media

# Expose internal port
EXPOSE 8080

# Execute the application natively
CMD ["./dipak_site"]
