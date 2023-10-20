# Build Stage
FROM rust:latest as builder

WORKDIR /usr/src/rs-short
COPY . .

# This will build the application in release mode
RUN cargo build --release

# Final Stage
FROM debian:latest

# Install necessary runtime dependencies (if any)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/rs-short/target/release/rs-short /usr/local/bin/rs-short

# Expose the application port
EXPOSE 3000

CMD ["rs-short"]
