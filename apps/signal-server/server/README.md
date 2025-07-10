# webrtc-signal-server

A general WebRTC signal server for device-to-device communication, written in Rust and powered by async networking and WebSockets.

## Features

- Simple WebSocket-based signaling for WebRTC devices
- Device registration and discovery
- Message relay between devices
- Asynchronous, scalable, and easy to deploy

## Usage

Add to your workspace or build as a standalone binary:

```sh
cargo build --release
```

Run the server (default port: 9000):

```sh
cargo run --release
```

The server listens for WebSocket connections on `0.0.0.0:9000`.

## Protocol

Clients communicate with the server using JSON messages:

### Register

```json
{ "type": "register", "device_id": "your-unique-id" }a
```

### List Devices

```json
{ "type": "list_devices" }
```

### Relay Message

```json
{ "type": "relay", "to": "target-device-id", "data": { ... } }
```

### Server Responses

- List of devices:
  ```json
  { "type": "devices", "devices": ["device1", "device2"] }
  ```
- Relayed message:
  ```json
  { "type": "relay", "from": "device1", "data": { ... } }
  ```
- Error:
  ```json
  { "type": "error", "error": "description" }
  ```

## License

MIT OR Apache-2.0

## Repository

[https://github.com/stars-labs/cypto-rust-tools](https://github.com/stars-labs/cypto-rust-tools)
