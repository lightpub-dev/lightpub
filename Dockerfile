# Build stage
FROM golang:1.23-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache git gcc musl-dev

# Copy go.mod and go.sum first to leverage caching
COPY go.mod go.sum* ./

# Use BuildKit cache mount to speed up module downloads
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go mod download

# Copy source code
COPY . .

# Build with cache mount for faster builds
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -v -o lightpub ./*.go

# Runtime stage
FROM alpine:3

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata openssl

# Copy the binary from the builder stage
COPY --from=builder /app/lightpub .

# Copy templates, static files, etc.
COPY templates ./templates
COPY static ./static
COPY scripts ./scripts

# Expose the port your application runs on
EXPOSE 8000

# Run the application
CMD ["./scripts/entrypoint.sh"]
