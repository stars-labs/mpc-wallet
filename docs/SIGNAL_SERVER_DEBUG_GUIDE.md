# Signal Server Debug Guide

This guide provides comprehensive instructions for debugging and monitoring the WebRTC signal server used by the MPC wallet.

## Quick Start

```bash
# Stop any existing server and start fresh with debug logging
./scripts/signal-server-debug.sh restart

# Monitor server health in real-time
./scripts/signal-server-monitor.sh monitor

# Watch logs with color highlighting
./scripts/signal-server-monitor.sh logs
```

## Available Scripts

### 1. signal-server-debug.sh

Main script for managing the signal server with enhanced debugging capabilities.

#### Commands:

```bash
# Start server with defaults (port 9000, debug logging)
./scripts/signal-server-debug.sh start

# Start with custom configuration
./scripts/signal-server-debug.sh start 8080              # Custom port
./scripts/signal-server-debug.sh start 9000 0.0.0.0 trace  # Trace logging

# Stop the server
./scripts/signal-server-debug.sh stop

# Restart (stop + start)
./scripts/signal-server-debug.sh restart

# Check server status
./scripts/signal-server-debug.sh status

# Monitor logs in real-time with color coding
./scripts/signal-server-debug.sh monitor

# Run in interactive debug mode (foreground with trace logging)
./scripts/signal-server-debug.sh debug

# Clean old log files
./scripts/signal-server-debug.sh clean
```

#### Log Levels:
- `error` - Only errors
- `warn` - Errors and warnings
- `info` - Informational messages
- `debug` - Debug messages (recommended for debugging)
- `trace` - Everything including detailed trace data
- `release` - Build in release mode for production

### 2. signal-server-monitor.sh

Real-time health monitoring and connection tracking.

#### Commands:

```bash
# Monitor server health (refreshes every 2 seconds)
./scripts/signal-server-monitor.sh monitor

# Test WebSocket connectivity
./scripts/signal-server-monitor.sh test localhost 9000

# Watch logs with optional filtering
./scripts/signal-server-monitor.sh logs          # All logs
./scripts/signal-server-monitor.sh logs error    # Only errors
./scripts/signal-server-monitor.sh logs connect  # Connection events

# Show connection statistics
./scripts/signal-server-monitor.sh stats
```

## Common Debug Scenarios

### Scenario 1: WebSocket Connection Reset Issues

**Symptoms:**
- "Connection reset without closing handshake" errors
- Clients disconnect immediately after connecting

**Debug Steps:**

1. **Stop all processes and start fresh:**
```bash
# Kill all signal server and TUI processes
pkill -f webrtc-signal-server
pkill -f mpc-wallet-tui

# Start server with trace logging
./scripts/signal-server-debug.sh start 9000 0.0.0.0 trace

# Monitor in another terminal
./scripts/signal-server-monitor.sh monitor
```

2. **Check for duplicate connections:**
```bash
# Watch for rapid connect/disconnect patterns
./scripts/signal-server-monitor.sh logs | grep -E "connected|disconnected"
```

3. **Verify no port conflicts:**
```bash
ss -tlnp | grep 9000
```

### Scenario 2: Participants Can't Find Each Other

**Symptoms:**
- DKG stuck at "Waiting for participants"
- Sessions not visible in join screen

**Debug Steps:**

1. **Monitor session announcements:**
```bash
# Watch for session creation and discovery
./scripts/signal-server-monitor.sh logs | grep -E "announce|session|discover"
```

2. **Check active connections:**
```bash
./scripts/signal-server-monitor.sh stats
```

3. **Test from each node:**
```bash
# Test WebSocket from each machine
./scripts/signal-server-monitor.sh test <server-ip> 9000
```

### Scenario 3: Performance Issues

**Symptoms:**
- Slow message delivery
- High memory/CPU usage

**Debug Steps:**

1. **Monitor resource usage:**
```bash
./scripts/signal-server-monitor.sh monitor
# Look at Memory and CPU sections
```

2. **Check for memory leaks:**
```bash
# Run for extended period and watch RSS memory
watch -n 5 'ps aux | grep webrtc-signal-server'
```

3. **Analyze connection patterns:**
```bash
# Look for connection leaks
./scripts/signal-server-monitor.sh stats
```

## Advanced Debugging

### Enable Maximum Verbosity

```bash
# Set all components to trace level
export RUST_LOG=trace,webrtc_signal_server=trace,tokio=trace,tungstenite=trace
export RUST_BACKTRACE=full

# Run in foreground for immediate feedback
./scripts/signal-server-debug.sh debug
```

### Capture Network Traffic

```bash
# Capture WebSocket traffic (requires tcpdump)
sudo tcpdump -i any -w signal-server.pcap port 9000

# Analyze with Wireshark
wireshark signal-server.pcap
```

### Test with Manual WebSocket Client

```bash
# Using websocat (install: cargo install websocat)
websocat ws://localhost:9000

# Send test messages
{"type":"register","device_id":"test-client"}
{"type":"get_sessions"}
```

### Log Analysis

```bash
# Find all errors in recent logs
grep -r "ERROR\|PANIC" logs/signal-server_*.log

# Count connection events
grep -c "connected" logs/signal-server_*.log

# Find specific device activity
grep "mpc-1" logs/signal-server_*.log

# Analyze timing patterns
grep "connected\|disconnected" logs/signal-server_*.log | \
  awk '{print $1}' | \
  uniq -c
```

## Log File Locations

- **Server logs:** `logs/signal-server_YYYYMMDD_HHMMSS.log`
- **Error logs:** `logs/signal-server_YYYYMMDD_HHMMSS_error.log`
- **Monitor logs:** `/tmp/signal-server-monitor.log`
- **PID file:** `/tmp/signal-server.pid`

## Environment Variables

```bash
# Control logging verbosity
export RUST_LOG=debug

# Enable backtrace for errors
export RUST_BACKTRACE=1

# Custom server configuration
export SIGNAL_SERVER_HOST=0.0.0.0
export SIGNAL_SERVER_PORT=9000
```

## Troubleshooting Tips

1. **Always restart the signal server** when debugging connection issues - it may have stale state
2. **Use trace logging** when investigating specific issues, but switch back to debug for normal operation
3. **Monitor from the start** - begin monitoring before starting the TUI nodes
4. **Check firewall rules** if nodes can't connect from different machines
5. **Clear browser cache** if using the browser extension - WebSocket connections can be cached

## Testing Checklist

Before running a DKG session:

- [ ] Signal server is freshly restarted
- [ ] Monitor shows server is healthy
- [ ] No errors in recent logs
- [ ] Port 9000 is accessible from all nodes
- [ ] All old TUI processes are killed
- [ ] Log level set appropriately (debug or trace)
- [ ] Monitor is running to track connections

## Quick Commands Reference

```bash
# Full reset and debug setup
pkill -f webrtc-signal-server && pkill -f mpc-wallet-tui
./scripts/signal-server-debug.sh start 9000 0.0.0.0 debug

# Terminal 1: Monitor health
./scripts/signal-server-monitor.sh monitor

# Terminal 2: Watch logs
./scripts/signal-server-monitor.sh logs

# Terminal 3-5: Start TUI nodes
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-1
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-2
cargo run --bin mpc-wallet-tui -- --signal-server ws://localhost:9000 --device-id mpc-3

# Check everything is connected
./scripts/signal-server-monitor.sh stats
```

## Common Error Messages and Solutions

| Error Message | Likely Cause | Solution |
|--------------|-------------|----------|
| "Connection reset without closing handshake" | Duplicate connections or server crash | Restart signal server |
| "Address already in use" | Port 9000 is occupied | Kill existing process or use different port |
| "Connection refused" | Server not running | Start signal server |
| "No route to host" | Firewall blocking | Check firewall rules |
| "Too many open files" | Resource limit reached | Increase ulimit: `ulimit -n 4096` |

## Performance Tuning

For production or high-load scenarios:

```bash
# Build in release mode
./scripts/signal-server-debug.sh start 9000 0.0.0.0 release

# Increase file descriptor limit
ulimit -n 65536

# Use optimized allocator
export MALLOC_ARENA_MAX=2

# Run with real-time priority (requires root)
sudo nice -n -20 ./scripts/signal-server-debug.sh start
```