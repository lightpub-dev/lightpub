# Syntax for BuildKit features
# Build stage
FROM rust:slim AS builder

ARG BUILD_TYPE="release"

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libgexiv2-dev \
    clang \
    lld \
    dos2unix \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty project for caching dependencies
RUN cargo new --bin app
WORKDIR /usr/src/app/app

# Copy only dependency files first
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./

# Copy the rest of the source code
COPY migration ./migration
COPY entity ./entity
COPY packages ./packages
COPY src ./src

# Build the application with cache
COPY build.sh .
RUN dos2unix build.sh
RUN chmod +x build.sh
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/app/target \
    bash ./build.sh ${BUILD_TYPE}

RUN --mount=type=cache,target=/usr/src/app/app/target cp target/${BUILD_TYPE}/lightpub_rs /usr/src/app/lightpub_rs

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    openssl \
    ca-certificates \
    libgexiv2-2 \
    && rm -rf /var/lib/apt/lists/*

# Create data directory
RUN mkdir -p data

# Copy static data
COPY templates ./templates
COPY static ./static

# copy scripts
COPY generate-jwt-keys.sh /app
COPY generate-vapid-keys.sh /app
COPY entrypoint.sh /app
RUN rm -f /app/.env && touch /app/.env

# Set environment variables
ENV JWT_PUBLIC_KEY_FILE=data/jwtpub.pem \
    JWT_SECRET_KEY_FILE=data/jwt.pem \
    RUST_LOG=info

# Create volume for persistent data
VOLUME ["/app/data", "/app/uploads"]
EXPOSE 8000/tcp

# Copy the built binary from builder
COPY --from=builder /usr/src/app/lightpub_rs ./lightpub_rs

ENTRYPOINT ["/app/entrypoint.sh"]
