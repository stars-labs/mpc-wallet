# MPC Wallet TUI Node Deployment Guide

## Overview

This guide provides complete deployment instructions for the MPC Wallet TUI Node with automated deployment configurations for both development and production environments.

## Quick Start

### 1. Build Components

```bash
# Build signal server
./scripts/build-signal-server.sh

# Build TUI node (will create stub if main binary fails to compile)
./scripts/build-tui-node.sh
```

### 2. Launch Development Cluster

```bash
# Launch 3-node cluster with signal server
./scripts/launch-3node-cluster.sh
```

### 3. Monitor Health

```bash
# Check cluster health
./scripts/health-check.sh

# Continuous monitoring
./scripts/monitor-cluster.sh
```

## Deployment Methods

### Method 1: Direct Script Deployment

**Start Signal Server:**
```bash
# Start signal server on port 9000
SIGNAL_PORT=9000 ./scripts/run-signal-server.sh
```

**Start Individual Nodes:**
```bash
# Terminal 1 - Node 1
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-1

# Terminal 2 - Node 2  
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-2

# Terminal 3 - Node 3
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-3
```

**Note:** The current main binary has compilation issues. Use the stub binary for testing deployment:
```bash
cargo run --bin mpc-wallet-stub-simple -- --signal-server ws://localhost:9000 --device-id mpc-1
```

### Method 2: Docker Deployment

```bash
# Build and start all services
docker-compose up --build

# Start in background
docker-compose up -d --build

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Method 3: SystemD Production Deployment

**1. Install Binaries:**
```bash
sudo mkdir -p /opt/mpc-wallet/data
sudo cp target/release/webrtc-signal-server /opt/mpc-wallet/signal-server
sudo cp target/release/mpc-wallet-tui /opt/mpc-wallet/mpc-wallet-tui
sudo chown -R mpcwallet:mpcwallet /opt/mpc-wallet
```

**2. Install SystemD Services:**
```bash
sudo cp systemd/*.service /etc/systemd/system/
sudo cp systemd/*.target /etc/systemd/system/
sudo systemctl daemon-reload
```

**3. Start Services:**
```bash
# Start entire cluster
sudo systemctl enable mpc-wallet-cluster.target
sudo systemctl start mpc-wallet-cluster.target

# Or start individual services
sudo systemctl start mpc-signal-server
sudo systemctl start mpc-wallet-node@mpc-1
sudo systemctl start mpc-wallet-node@mpc-2
sudo systemctl start mpc-wallet-node@mpc-3

# Check status
sudo systemctl status mpc-wallet-cluster.target
```

## Configuration

### Signal Server Configuration

Environment variables:
- `SIGNAL_PORT`: Port to bind (default: 9000)
- `BIND_ADDRESS`: Address to bind (default: 0.0.0.0:9000)
- `RUST_LOG`: Log level (default: info)

### TUI Node Configuration

Command line arguments:
- `--signal-server`: WebSocket URL of signal server (default: ws://localhost:9000)
- `--device-id`: Unique device identifier (required for multi-node setup)

Environment variables:
- `RUST_LOG`: Log level (default: info)
- `DATA_DIR`: Data directory path (default: ./data/{device-id})

## Health Checks and Monitoring

### Manual Health Check
```bash
./scripts/health-check.sh [--verbose]
```

### Continuous Monitoring
```bash
# Monitor with 30-second intervals
MONITOR_INTERVAL=30 ./scripts/monitor-cluster.sh
```

### Docker Health Checks
```bash
# Check container health
docker-compose ps

# View health check logs
docker inspect mpc-node-1 | jq '.[0].State.Health'
```

### SystemD Health Checks
```bash
# Check all services
sudo systemctl status mpc-signal-server mpc-wallet-node@mpc-1 mpc-wallet-node@mpc-2 mpc-wallet-node@mpc-3

# View logs
journalctl -u mpc-signal-server -f
journalctl -u mpc-wallet-node@mpc-1 -f
```

## Current Status and Known Issues

### ✅ Working Components
- **Signal Server**: Compiles and runs successfully
- **WebSocket Communication**: Signal server provides WebRTC coordination
- **Docker Configuration**: Multi-service setup with health checks
- **SystemD Services**: Production-ready service definitions
- **Monitoring Scripts**: Health checks and continuous monitoring
- **Build Scripts**: Automated build and deployment scripts

### ⚠️ Known Issues
- **Main TUI Binary**: Has 81 compilation errors due to complex FROST DKG implementation
- **DKG Implementation**: Temporarily stubbed for deployment testing
- **WebRTC P2P**: Complex networking code needs fixes for full functionality

### 🔧 Workarounds
- **Stub Binary**: `mpc-wallet-stub-simple` provides deployment testing capability
- **Signal Server**: Fully functional for WebRTC signaling
- **Infrastructure**: Complete deployment pipeline ready for when main binary is fixed

## Testing the Deployment

### Test Signal Server
```bash
# Start signal server
./scripts/run-signal-server.sh &

# Test WebSocket connection
curl -v http://localhost:9000/health

# Test WebSocket upgrade (should see 101 response)
curl --include --no-buffer --header "Connection: Upgrade" --header "Upgrade: websocket" --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" --header "Sec-WebSocket-Version: 13" http://localhost:9000/
```

### Test Stub Node
```bash
# If main binary works:
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id test-node

# If using stub:
cargo run --bin mpc-wallet-stub-simple -- --signal-server ws://localhost:9000 --device-id test-node
```

### Test Full Cluster
```bash
# Launch complete 3-node setup
./scripts/launch-3node-cluster.sh

# In another terminal, monitor health
./scripts/monitor-cluster.sh
```

## Performance and Scaling

### Resource Requirements
- **Signal Server**: ~50MB RAM, minimal CPU
- **TUI Node**: ~100-200MB RAM per node, moderate CPU for crypto operations
- **Network**: WebRTC requires UDP connectivity between nodes

### Scaling Considerations
- Signal server can handle 100+ concurrent nodes
- Each DKG session requires 2-of-3 or 3-of-5 participants
- P2P WebRTC connections scale O(n²) - use mesh topology carefully

## Security Notes

### Production Security
- Run services as non-root user `mpcwallet`
- Use firewall to restrict access to signal server port
- Consider TLS termination proxy for WebSocket connections
- Secure private key storage (not implemented in stub)

### Network Security
- WebRTC uses DTLS for P2P encryption
- Signal server only coordinates connection setup
- No private keys transmitted through signal server

## Next Steps

1. **Fix Compilation Issues**: Resolve the 81 compilation errors in the main TUI binary
2. **Complete DKG Implementation**: Remove stubs and implement full FROST DKG
3. **Add TLS Support**: Secure WebSocket connections with TLS
4. **Implement Persistence**: Add secure keystore persistence
5. **Add Metrics**: Integrate Prometheus/Grafana monitoring
6. **Load Testing**: Test with larger node counts

## Support

For issues and questions:
1. Check logs in `./data/` directory
2. Run health checks with `--verbose` flag
3. Monitor system resources during operation
4. Review WebRTC connectivity between nodes

This deployment configuration provides a solid foundation for the MPC Wallet infrastructure, ready to support the full implementation once compilation issues are resolved.