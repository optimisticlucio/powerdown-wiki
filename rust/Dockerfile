FROM lukemathwalker/cargo-chef:latest-rust-1.89.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Now we install some dependencies
RUN apt-get update && \
    # Add ca-certificates for reqwest
    apt-get install -y ca-certificates curl gnupg lsb-release && \
    # Add PostgreSQL repository
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/postgresql-keyring.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/postgresql-keyring.gpg] http://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list && \
    apt-get update && \
    # Install postgresql-client-16, for pg_dump
    apt-get install -y postgresql-client-16 && \
    # Clean up
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/powerdown-wiki /usr/local/bin
COPY ./static /app/static
ENTRYPOINT ["stdbuf", "-oL", "/usr/local/bin/powerdown-wiki"]
