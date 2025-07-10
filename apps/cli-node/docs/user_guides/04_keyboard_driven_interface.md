# Keyboard-Driven Interface Guide

The FROST MPC CLI Node now features a modern keyboard-driven interface with popup windows for all major functions. This guide explains how to use the new interface effectively.

## Quick Start

Instead of typing commands, you can now access all functionality through keyboard shortcuts and interactive popup windows.

## Keyboard Shortcuts (Normal Mode)

### Primary Functions
- **`m` or `M`** - Open Main Menu (access all functions from here)
- **`p` or `P`** - Create/Propose new session (quick access)
- **`w` or `W`** - View wallet list
- **`a` or `A`** - Accept session invitations
- **`t` or `T`** - Sign transaction (opens chain selection)
- **`Tab`** - View pending signing requests

### Navigation & Utility
- **`↑/↓`** - Scroll through log messages
- **`?`** - Show help popup
- **`s`** - Save log to file
- **`q`** - Quit application
- **`i`** - Enter legacy command mode (for typing commands)
- **`o`** - Quick accept first pending session

## Popup Windows

### Main Menu (`m`)
The main menu provides access to all functionality:
1. Create/Join Session
2. List Wallets
3. Sign Transaction
4. Accept Session
5. View Help

Navigate with `↑/↓` arrows, select with `Enter`, close with `Esc`.

### Session Proposal Popup (`p`)
Create a new session with an interactive form:
- Session Name
- Total Participants
- Threshold
- Participant List (comma-separated)

Use `Tab` to move between fields, type to enter values, `Enter` to submit, `Esc` to cancel.

### Wallet List Popup (`w`)
Displays all available wallets with their details. Navigate with `↑/↓`, close with `Esc`.

### Accept Session Popup (`a`)
Shows all pending session invitations with details:
- Session ID
- Type (DKG or Signing)
- Initiator
- Participants

Navigate with `↑/↓`, accept with `Enter`, close with `Esc`.

### Sign Transaction Popup (`t`)
Interactive signing interface with blockchain selection:
1. Select blockchain (Ethereum, BSC, Polygon, etc.)
2. Enter transaction data
3. Submit signing request

Use `↑/↓` to select chain, `Tab` to edit transaction data, `Enter` to sign.

### Signing Requests Popup (`Tab`)
View and accept pending signing requests from other participants. Shows:
- Signing ID
- Requesting device
- Transaction preview

Navigate with `↑/↓`, accept with `Enter`, close with `Esc`.

## Workflow Examples

### Creating a New Wallet (DKG Session)
1. Press `p` to open session proposal
2. Enter session details:
   - Name: `wallet_2of3`
   - Total: `3`
   - Threshold: `2`
   - Participants: `mpc-1,mpc-2,mpc-3`
3. Press `Enter` to create
4. Other participants press `a` to view and accept

### Signing a Transaction
1. Press `t` to open signing interface
2. Select blockchain with `↑/↓`
3. Press `Tab` to enter transaction data
4. Type or paste transaction hex
5. Press `Enter` to initiate signing
6. Other participants see popup via `Tab` key

### Quick Session Accept
- Press `o` to accept the first pending invitation
- Or press `a` to see all invitations and choose

## Legacy Command Mode

For users who prefer typing commands, press `i` to enter command mode:
- `/list` - List connected devices
- `/wallets` - Show wallets
- `/propose` - Create session
- `/accept` - Accept invitation
- `/sign` - Sign transaction

Press `Esc` to exit command mode.

## Tips

1. **Faster Navigation**: Use direct shortcuts (`p`, `w`, `a`, `t`) instead of going through the main menu
2. **Multiple Sessions**: The interface handles multiple pending invitations gracefully
3. **Visual Feedback**: Selected items are highlighted in yellow
4. **Context Aware**: Popups show relevant information based on current state
5. **No Mouse Required**: Everything is keyboard accessible

## Advantages Over Command-Based Interface

1. **Discoverable**: All options visible in menus
2. **Fewer Errors**: Interactive forms validate input
3. **Faster**: Direct keyboard shortcuts
4. **Visual**: See all options at once
5. **Intuitive**: Standard navigation patterns (arrows, Enter, Esc)

## Troubleshooting

- If a popup seems stuck, press `Esc` to close it
- If shortcuts don't work, ensure you're in Normal mode (not Input mode)
- Check the status bar for current mode indication
- Use `?` to see all available shortcuts at any time