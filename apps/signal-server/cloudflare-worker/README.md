# WebRTC Signal Server - Cloudflare Worker

A simple, scalable WebRTC signaling server implemented as a Cloudflare Worker using Rust and Durable Objects.  
This server allows devices to register, list, and relay messages for device-to-device WebRTC connections.

## Features

- **WebSocket-based signaling** for WebRTC device discovery and message relay.
- **Durable Object** for scalable, consistent device management.
- Compatible with Cloudflare's free plan (uses SQLite-backed Durable Objects).

## Protocol

- **Register:**  
  `{ "type": "register", "device_id": "<your_id>" }`
- **List Devices:**  
  `{ "type": "list_devices" }`
- **Relay:**  
  `{ "type": "relay", "to": "<device_id>", "data": <any JSON> }`

### Server Messages

- **Devices List:**  
  `{ "type": "devices", "devices": [ ... ] }`
- **Relay:**  
  `{ "type": "relay", "from": "<device_id>", "data": <any JSON> }`
- **Error:**  
  `{ "type": "error", "error": "<message>" }`

## Project Structure

- `src/lib.rs` — Main Worker and Durable Object logic.
- `wrangler.toml` — Cloudflare Worker and Durable Object configuration.

## Deploying to Cloudflare

1. **Install Wrangler:**  
   ```sh
   npm install -g wrangler
   # or for v4+
   npm install --save-dev wrangler@4
   ```

2. **Configure Durable Object in `wrangler.toml`:**
   ```toml
   [durable_objects]
   bindings = [{ name = "Devices", class_name = "Devices" }]

   [[migrations]]
   tag = "v1"
   new_sqlite_classes = ["Devices"]
   ```

3. **Build the Worker:**
   ```sh
   wrangler build
   ```

4. **Publish/Deploy:**
   ```sh
   wrangler deploy
   # or for older versions:
   wrangler publish
   ```

5. **Access your Worker:**  
   Wrangler will output your deployed URL. Connect your WebRTC clients to this endpoint.

## Notes

- **Durable Objects** are required for device state.  
- On the free plan, you must use `new_sqlite_classes` in your migration.
- See [Cloudflare Durable Objects Docs](https://developers.cloudflare.com/workers/learning/using-durable-objects/) for more info.

---

**Happy hacking!**
