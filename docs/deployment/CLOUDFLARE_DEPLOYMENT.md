# Cloudflare Workers Deployment Guide

## Prerequisites
- Cloudflare account with Workers enabled
- `wrangler` CLI installed (`npm install -g wrangler`)
- Durable Objects enabled on your account

## Deployment Steps

```bash
# 1. Navigate to Cloudflare Worker directory
cd apps/signal-server/cloudflare-worker

# 2. Login to Cloudflare (if not already)
wrangler login

# 3. Configure wrangler.toml with your account ID
# Edit wrangler.toml and set your account_id

# 4. Deploy the Worker
wrangler deploy

# 5. Test the deployment
# The worker will be available at:
# https://your-worker-name.your-subdomain.workers.dev
```

## Configuration

Edit `wrangler.toml`:

```toml
name = "mpc-wallet-signal-server"
main = "src/lib.rs"
compatibility_date = "2024-01-01"

[[durable_objects.bindings]]
name = "Devices"
class_name = "Devices"
script_name = "mpc-wallet-signal-server"

[build]
command = "cargo install -q worker-build && worker-build --release"
```

## Features

### Device-Bound Sessions
- Sessions are automatically removed when creator disconnects
- All participants receive immediate notification
- No orphaned sessions in the system

### Stateless Rejoin
- Devices can query their active sessions on reconnect
- No local persistence required
- Server maintains minimal state

### Message Types Supported
- `Register` - Register device with server
- `ListDevices` - Get list of connected devices
- `Relay` - Relay messages between devices
- `AnnounceSession` - Create new session (bound to creator)
- `QueryMyActiveSessions` - Get sessions where device is participant
- `RequestActiveSessions` - Legacy session discovery

### Notifications
- `SessionsForDevice` - Response to QueryMyActiveSessions
- `SessionRemoved` - Sent when session creator disconnects
- `Devices` - Updated device list
- `SessionAvailable` - New session announced

## Testing

```bash
# Test locally with miniflare
wrangler dev

# Tail production logs
wrangler tail

# View Durable Object storage
wrangler tail --format pretty
```

## Monitoring

The Worker logs important events:
- Device registration/disconnection
- Session creation/removal
- Message relay operations

Check logs with:
```bash
wrangler tail --format json | jq '.logs[].message'
```