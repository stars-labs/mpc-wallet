# Multi-stage Dockerfile for MPC Wallet TUI Node
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all source code
COPY . .

# Build the release binary
RUN cargo build --release --bin mpc-wallet-tui

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r mpcwallet && useradd -r -g mpcwallet mpcwallet

# Create directories
RUN mkdir -p /opt/mpc-wallet/data \
    && chown -R mpcwallet:mpcwallet /opt/mpc-wallet

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mpc-wallet-tui /opt/mpc-wallet/mpc-wallet-tui

# Set ownership and permissions
RUN chown mpcwallet:mpcwallet /opt/mpc-wallet/mpc-wallet-tui \
    && chmod +x /opt/mpc-wallet/mpc-wallet-tui

# Switch to non-root user
USER mpcwallet

# Set working directory
WORKDIR /opt/mpc-wallet

# Expose any ports (none needed for this TUI app)
# EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD pgrep mpc-wallet-tui || exit 1

# Default command
ENTRYPOINT ["/opt/mpc-wallet/mpc-wallet-tui"]
CMD ["--help"]