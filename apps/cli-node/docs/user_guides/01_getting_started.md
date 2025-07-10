# MPC CLI Node Usage Guide

This guide explains how to use the `frost-mpc-cli-node` application for participating in MPC wallet creation and signing sessions via a signaling server.

## Running the CLI Node

1.  **Start the Signaling Server:** Ensure the WebRTC signaling server is running.
    ```bash
    cargo run -p webrtc-signal-server
    ```

2.  **Run the CLI Node:** Open a new terminal for each participant and run the following command with required parameters:
    ```bash
    cargo run -p frost-mpc-cli-node -- --device-id <your-id> --curve <secp256k1|ed25519>
    ```
    
    Example:
    ```bash
    cargo run -p frost-mpc-cli-node -- --device-id mpc-1 --curve secp256k1
    ```

    Parameters:
    - `--device-id`: A unique identifier for this node (e.g., `mpc-1`, `mpc-2`)
    - `--curve`: Cryptographic curve to use (`secp256k1` for Ethereum or `ed25519` for Solana)
    - `--webrtc`: Optional WebRTC signaling server URL (defaults to `wss://auto-life.tech`)

## TUI Interface

The CLI node presents a terminal user interface (TUI) with the following sections:

1.  **Device ID:** Displays the unique ID for this node and the selected curve.
2.  **Devices:** Lists other devices currently connected to the signaling server and shows their WebRTC connection states.
3.  **Log:** Shows recent events, received messages, and errors.
4.  **Session/Invites/DKG/Signing:**
    - **Session:** Displays information about the current MPC session if joined.
    - **Invites:** Lists pending session invites by `session_id`.
    - **DKG Status:** Shows the current status of the Distributed Key Generation process.
    - **Mesh Status:** Indicates if all devices have established WebRTC connections.
    - **Signing Status:** Shows the current status of any active signing process.
    - **Input:** Shows the input prompt `>` when in input mode.

## Commands and Keybindings

### Normal Mode (Default)

- `i`: Enter **Input Mode** to type commands.
- `o`: Accept the *first* pending session invite listed under "Invites". If no invites are pending, a message will be logged.
- `q`: Quit the application.

### Input Mode (Activated by `i`)

Type commands starting with `/` and press `Enter`.

- `/list`: Manually request an updated list of devices from the server. The list also updates periodically and on device join/disconnect.
- `/propose <session_id> <total> <threshold> <device1,device2,...>`: Propose a new MPC session.
  - `<session_id>`: A unique name for the session (e.g., `mywallet`).
  - `<total>`: The total number of participants required (e.g., `3`).
  - `<threshold>`: The signing threshold (e.g., `2`).
  - `<device1,device2,...>`: A comma-separated list of the exact `device_id`s of all participants (including yourself).
  - *Example:* `/propose mywallet 3 2 mpc-1,mpc-2,mpc-3`
- `/join <session_id>`: Join an existing session you were invited to.
  - *Example:* `/join mywallet`
- `/accept <session_id>`: (Alternative to 'o') Accept a specific pending session proposal by its `session_id`.
  - *Example:* `/accept mywallet`
- `/relay <target_device_id> <json_data>`: Send an arbitrary JSON message to another device via the signaling server.
  - `<target_device_id>`: The exact `device_id` of the recipient.
  - `<json_data>`: A valid JSON object or value (e.g., `{"type":"hello","value":123}`).
  - *Example:* `/relay mpc-2 {"type":"ping","payload":{"id":1}}`
- `/send <target_device_id> <message>`: Send a direct WebRTC message to another device.
  - `<target_device_id>`: The exact `device_id` of the recipient.
  - `<message>`: Any text message you want to send directly to the device.
  - *Example:* `/send mpc-2 Hello, this is a direct message!`
- `/status`: Show detailed information about the current session and mesh state.
- `/mesh_ready`: Manually indicate this node is ready with all WebRTC connections established.
- `/sign <transaction_data>`: Initiate a threshold signing process with the specified transaction data.
  - `<transaction_data>`: The data to be signed (typically hex-encoded).
  - *Example:* `/sign 0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef`
- `/acceptSign <signing_id>`: Accept a signing request.
  - `<signing_id>`: The identifier of the signing request to accept.
  - *Example:* `/acceptSign sign_mpc-1_1624511234`
- `Esc`: Exit Input Mode without sending a command.
- `Backspace`: Delete the last character typed.

## Understanding WebRTC Connection Status

After participants have agreed to join a session, the WebRTC connection establishment process begins:

1. **Signaling Exchange:** Devices exchange WebRTC signaling data (SDP offers/answers, ICE candidates) via the signaling server.
   - The log will show messages about offers, answers, and ICE candidates being sent and received.

2. **Connection States:** In the **Devices** section, you'll see connection status indicators:
   - `New`: Initial state
   - `Connecting`: Connection attempt in progress
   - `Connected`: WebRTC connection established
   - `Failed`: Connection attempt failed
   - `Disconnected`: Connection was established but then lost

3. **Data Channel Status:** For successful MPC communication, data channels must be opened.
   - TUI Log will show: `Data channel opened with mpc-2`
   - Console may show: `WebRTC data channel state change: open`

4. **Mesh Readiness:** A complete mesh is formed when all participants have established WebRTC connections with each other.
   - Each node automatically sends a `channel_open` message when a data channel is successfully opened
   - When all required connections are established, a node automatically signals `mesh_ready` to all devices
   - Manual intervention with `/mesh_ready` command is only needed if the automatic process fails
   - The TUI will show "Mesh Status: Ready" when all devices report readiness
   - When all devices report mesh readiness, the MPC protocol proceeds automatically

## Distributed Key Generation (DKG)

Once the full WebRTC mesh is established and all participants have signaled readiness, the DKG process begins:

1. **Round 1 (Commitments):** Each node generates and shares its cryptographic commitments with all others.
   - Log will show: `Initiating DKG Round 1...`
   - DKG Status changes to: `Round 1 In Progress`
   
2. **Round 2 (Shares):** After all commitments are received, nodes exchange encrypted key shares.
   - Log will show: `All Round 1 packages received, proceeding to Round 2...`
   - DKG Status changes to: `Round 2 In Progress`
   
3. **Finalization:** Nodes verify received shares and compute their final key shares.
   - Log will show: `All Round 2 packages received, finalizing DKG...`
   - DKG Status changes to: `Finalizing` then `DKG Complete`
   - The node will display the group public key for the newly created wallet

The "DKG Status" in the TUI updates through these phases:
`Idle` → `Round 1 In Progress` → `Round 1 Complete` → `Round 2 In Progress` → `Round 2 Complete` → `Finalizing` → `DKG Complete`

## Threshold Signing Process

After a successful DKG, the wallet can be used for threshold signing:

1. **Initiation:** Any participant can initiate a signing request with transaction data:
   - Use the command: `/sign <transaction_data>`
   
2. **Acceptance:** Other participants can accept the signing request:
   - Use the command: `/acceptSign <signing_id>` or press `o` if it's the first invite
   - The initiator needs to gather acceptances from at least the threshold number of participants

3. **Commitment Phase:** Once enough participants accept, the FROST signing protocol begins:
   - Selected signers exchange commitment values
   - Status shows: `Commitment Phase (sign_id): x/y commitments`

4. **Share Phase:** After all commitments are received, signature shares are generated:
   - Participants exchange signature shares
   - Status shows: `Share Phase (sign_id): x/y shares`

5. **Aggregation:** When all shares are collected, the signature is aggregated and verified:
   - Final signature is displayed in the log
   - Status changes to: `Complete (sign_id)`

The "Signing Status" in the TUI updates through these phases:
`Idle` → `Awaiting Acceptance` → `Commitment Phase` → `Share Phase` → `Complete`

## Example Workflow (Creating a 2-of-3 MPC Wallet and Signing)

This example shows how to set up a session for 3 participants (`mpc-1`, `mpc-2`, `mpc-3`) where any 2 are required to sign (`threshold = 2`).

1. **Start Server:**
   ```bash
   cargo run -p webrtc-signal-server
   ```

2. **Start Nodes:** In three separate terminals:
   ```bash
   cargo run -p frost-mpc-cli-node -- --device-id mpc-1 --curve secp256k1
   cargo run -p frost-mpc-cli-node -- --device-id mpc-2 --curve secp256k1
   cargo run -p frost-mpc-cli-node -- --device-id mpc-3 --curve secp256k1
   ```

3. **Session Proposal:**
   - On `mpc-1`, enter input mode (`i`) and type:
   ```
   /propose wallet_2of3 3 2 mpc-1,mpc-2,mpc-3
   ```
   - On `mpc-2` and `mpc-3`, press `o` to accept the proposal

4. **WebRTC Connection Establishment and DKG:**
   - The nodes will automatically establish WebRTC connections
   - When all connections are ready, DKG will automatically start
   - Watch the DKG Status progress through the rounds
   - When complete, all nodes will display the same group public key

5. **Signing a Transaction:**
   - On `mpc-1`, enter input mode (`i`) and type:
   ```
   /sign 0x123456789abcdef
   ```
   - On `mpc-2`, press `o` to accept the signing request
   - The signing process will proceed automatically through commitment and share phases
   - All nodes will receive the final signature when complete

## Troubleshooting

* **Device not showing in list:** Try using `/list` to refresh the device list.
* **Failed WebRTC connections:** Check your network settings. WebRTC may be blocked by some firewalls.
* **DKG fails to complete:** Ensure all nodes have proper WebRTC connections before starting DKG.
* **Mesh formation issues:** If the mesh isn't completing automatically, check the logs for connection errors and try the manual `/mesh_ready` command.
* **Message not received:** Verify that all participants have joined the same session ID.
* **State synchronization issues:** Sometimes restarting the affected nodes may help resolve state inconsistencies.
* **Signing process stalls:** Check that enough participants have accepted the signing request and that all selected signers can communicate.

For persistent issues, check the detailed logs to identify the specific failure point in the protocol flow.