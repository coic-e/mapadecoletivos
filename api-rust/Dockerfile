# Build stage
FROM rust:1.75-alpine as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apk add --no-cache musl-dev postgresql-dev openssl-dev

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache libpq openssl

WORKDIR /app

# Copy built binary from builder
COPY --from=builder /usr/src/app/target/release/mapadecoletivos-api-rust /app/

# Copy migrations
COPY migrations ./migrations

# Create uploads directory
RUN mkdir -p /app/uploads

# Expose port
EXPOSE 8080

CMD ["./mapadecoletivos-api-rust"]
