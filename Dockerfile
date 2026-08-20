# Build stage — build context must be the workspace root
FROM rust:1.97-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY api-rust/Cargo.toml api-rust/Cargo.toml
COPY db-types/Cargo.toml db-types/Cargo.toml
COPY api-types/Cargo.toml api-types/Cargo.toml

# Create dummy sources to build dependencies (cached layer)
RUN mkdir -p api-rust/src db-types/src api-types/src \
    && echo "fn main() {}" > api-rust/src/main.rs \
    && touch api-rust/src/lib.rs db-types/src/lib.rs api-types/src/lib.rs

RUN cargo build --release -p api-rust \
    && rm -rf api-rust/src db-types/src api-types/src

# Copy source code
COPY api-rust api-rust
COPY db-types db-types
COPY api-types api-types

# Build application. Remove the workspace crates' cached artifacts first so
# cargo rebuilds the real code instead of reusing the dummies from the deps layer.
RUN rm -rf target/release/api-rust \
        target/release/deps/api_rust-* target/release/deps/libapi_rust-* \
        target/release/deps/db_types-* target/release/deps/libdb_types-* \
        target/release/deps/api_types-* target/release/deps/libapi_types-* \
        target/release/.fingerprint/api-rust-* \
        target/release/.fingerprint/db-types-* \
        target/release/.fingerprint/api-types-* \
    && cargo build --release -p api-rust

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Usuário sem privilégio: se algum dia a API for explorada, o processo não
# tem root dentro do container — e o diretório de uploads é a única coisa que
# ele consegue escrever.
RUN useradd --system --create-home --uid 10001 ravemap

# Copy built binary from builder
COPY --from=builder /app/target/release/api-rust /app/

# Copy migrations
COPY api-rust/migrations ./migrations

# Create uploads directory
RUN mkdir -p /app/uploads \
    && chown -R ravemap:ravemap /app \
    && chmod 555 /app/api-rust

USER ravemap

# Expose port
EXPOSE 8080

CMD ["./api-rust"]
