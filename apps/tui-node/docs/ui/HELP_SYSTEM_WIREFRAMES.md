# Help System Wireframes

This document contains wireframes for the comprehensive help system, including context-sensitive help, tutorials, and command reference.

## Main Help Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MPC WALLET HELP CENTER                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Previous Screen                          Press ? anywhere for help│
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Help Topics:                                                    │     │
│   │                                                                   │     │
│   │  Getting Started                                                 │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ > [1] Quick Start Guide                                  │   │     │
│   │  │   [2] Understanding MPC Wallets                          │   │     │
│   │  │   [3] First Time Setup                                   │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Operations                                                      │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │   [4] Creating a New Wallet (DKG)                        │   │     │
│   │  │   [5] Joining a Session                                  │   │     │
│   │  │   [6] Signing Transactions                               │   │     │
│   │  │   [7] Managing Wallets                                   │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Advanced Topics                                                 │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │   [8] Offline Mode Operations                            │   │     │
│   │  │   [9] Import/Export Procedures                           │   │     │
│   │  │   [A] Security Best Practices                            │   │     │
│   │  │   [B] Troubleshooting                                    │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Reference                                                       │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │   [C] Keyboard Shortcuts                                 │   │     │
│   │  │   [D] Command Reference                                  │   │     │
│   │  │   [E] Glossary                                           │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓/1-9/A-E] Select Topic  [Enter] View  [/] Search  [Esc] Close          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Context-Sensitive Help Overlay

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CREATE DKG SESSION                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Curve Selection                    [Online Mode] [secp256k1]     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Configure new wallet session:                                   │     │
│   │  ┌────────────────────────────┐                                 │     │
│   │  │        HELP OVERLAY        │────────────────────────┐        │     │
│   │  │                            │                        │        │     │
│   │  │ Total Participants:        │                        │        │     │
│   │  │ The total number of devices│                        │        │     │
│   │  │ that will share the wallet │ [3]  (2-10)           │        │     │
│   │  │ key. Each gets a key share.│                        │        │     │
│   │  │                            │                        │        │     │
│   │  │ Threshold:                 │                        │        │     │
│   │  │ Minimum signatures needed  │ [2]  (required)       │        │     │
│   │  │ to authorize transactions. │                        │        │     │
│   │  │ Must be ≤ total participants                        │        │     │
│   │  │                            │                        │        │     │
│   │  │ Example: 2-of-3 means any │ipant IDs:             │        │     │
│   │  │ 2 participants can sign.   │───────────────────────┐        │     │
│   │  │                            │01 (You)              │        │     │
│   │  │ [Page 1/2] [→] More        │02________            │        │     │
│   │  └────────────────────────────┘03________            │        │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Validation: ✓ All fields valid                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [?] Toggle Help  [→] Next Page  [Esc] Close Help                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start Guide

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         QUICK START GUIDE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Help Center                                      Page 1 of 5      │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Welcome to MPC Wallet!                                          │     │
│   │                                                                   │     │
│   │  This guide will help you create your first threshold wallet    │     │
│   │  in just a few minutes.                                          │     │
│   │                                                                   │     │
│   │  What You'll Need:                                               │     │
│   │  • 2 or more devices (computers, phones, or hardware wallets)   │     │
│   │  • Network connection (or offline setup for enhanced security)  │     │
│   │  • 5-10 minutes for the setup process                           │     │
│   │                                                                   │     │
│   │  Step 1: Choose Your Setup                                       │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Scenario              │ Recommended Configuration       │   │     │
│   │  ├─────────────────────┼───────────────────────────────┤   │     │
│   │  │ Personal Wallet      │ 2-of-3 (phone, laptop, backup)│   │     │
│   │  │ Family Wallet        │ 2-of-4 (family members)       │   │     │
│   │  │ Business Treasury    │ 3-of-5 (board members)        │   │     │
│   │  │ High Security        │ 5-of-7 (offline, distributed) │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Step 2: Start the Process                                       │     │
│   │  1. On the coordinator device, select "Create New Wallet"        │     │
│   │  2. Choose your blockchain (Ethereum or Solana)                  │     │
│   │  3. Set participants and threshold                               │     │
│   │                                                                   │     │
│   │  [Try Interactive Demo]              [→] Continue to Step 3      │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [←→] Navigate  [D] Demo  [P] Print  [Esc] Back to Help                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts Reference

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KEYBOARD SHORTCUTS                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Help Center                              Quick Reference Card     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Global Shortcuts (Available Everywhere)                         │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Key     │ Action                                         │   │     │
│   │  ├─────────┼────────────────────────────────────────────────┤   │     │
│   │  │ ?       │ Show/hide context help                         │   │     │
│   │  │ Esc     │ Go back / Cancel operation                     │   │     │
│   │  │ Ctrl+C  │ Quit application (with confirmation)           │   │     │
│   │  │ Ctrl+L  │ Clear and redraw screen                        │   │     │
│   │  │ Ctrl+S  │ Save current state                             │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Navigation                                                      │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ ↑/↓     │ Move up/down in lists and menus                │   │     │
│   │  │ ←/→     │ Move left/right in forms                       │   │     │
│   │  │ PgUp/Dn │ Page up/down in long lists                     │   │     │
│   │  │ Home/End│ Jump to start/end of list                      │   │     │
│   │  │ Tab     │ Next field / Next section                      │   │     │
│   │  │ S-Tab   │ Previous field / Previous section              │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Quick Actions                                                   │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ M       │ Open main menu                                 │   │     │
│   │  │ W       │ View wallet list                               │   │     │
│   │  │ P       │ Propose/create new session                     │   │     │
│   │  │ J       │ Join existing session                          │   │     │
│   │  │ S       │ Start signing process                          │   │     │
│   │  │ I       │ Import wallet                                  │   │     │
│   │  │ E       │ Export wallet                                  │   │     │
│   │  │ R       │ Refresh current view                           │   │     │
│   │  │ L       │ View logs                                      │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [P] Print Reference  [E] Export as PDF  [Esc] Back                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Interactive Tutorial

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INTERACTIVE TUTORIAL - DKG PROCESS                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tutorial Progress: Step 3 of 8                              [Sandbox Mode] │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Step 3: Setting Threshold Parameters                            │     │
│   │                                                                   │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │    👆 Try it yourself!                                   │   │     │
│   │  │         ↓                                                │   │     │
│   │  │  Total Participants:    [_]  ← Enter 3 here             │   │     │
│   │  │                                                          │   │     │
│   │  │  Threshold:            [_]  ← Enter 2 here             │   │     │
│   │  │                                                          │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  💡 Tutorial Tip:                                                │     │
│   │  A 2-of-3 setup means any 2 participants can sign together.    │     │
│   │  This provides both security and availability - even if one     │     │
│   │  device is lost or offline, you can still access your funds.    │     │
│   │                                                                   │     │
│   │  Common Configurations:                                          │     │
│   │  • 2-of-3: Good balance of security and convenience            │     │
│   │  • 3-of-5: For business use with multiple stakeholders         │     │
│   │  • 5-of-7: Maximum security for high-value storage             │     │
│   │                                                                   │     │
│   │  ✓ Great job! You've configured a 2-of-3 wallet.               │     │
│   │                                                                   │     │
│   │  [←] Previous Step  [→] Next Step  [S] Skip Tutorial           │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Tutorial Mode: Changes are not saved           Time in tutorial: 2:34      │
├─────────────────────────────────────────────────────────────────────────────┤
│ [←→] Navigate Steps  [R] Restart  [H] Hint  [Esc] Exit Tutorial           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Troubleshooting Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TROUBLESHOOTING GUIDE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Help Center                              Search: connection_      │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Common Issues and Solutions:                                    │     │
│   │                                                                   │     │
│   │  🔍 Search Results for "connection"                              │     │
│   │                                                                   │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ > Connection timeout to signaling server                 │   │     │
│   │  │   Problem: Can't connect to wss://auto-life.tech        │   │     │
│   │  │                                                          │   │     │
│   │  │   Solutions:                                             │   │     │
│   │  │   1. Check internet connection                          │   │     │
│   │  │   2. Verify firewall allows WebSocket (port 443)       │   │     │
│   │  │   3. Try alternative server: wss://backup.signal.com    │   │     │
│   │  │   4. Use offline mode if network restricted             │   │     │
│   │  │                                                          │   │     │
│   │  │   [Run Network Diagnostic]  [View Detailed Guide]       │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │   WebRTC connection fails between participants          │   │     │
│   │  │   Problem: Peers can't establish direct connection      │   │     │
│   │  │                                                          │   │     │
│   │  │   Solutions:                                             │   │     │
│   │  │   1. Ensure all participants are in same session        │   │     │
│   │  │   2. Check NAT type (run diagnostic)                    │   │     │
│   │  │   3. Enable STUN/TURN servers in settings              │   │     │
│   │  │   4. Try relay mode if direct connection fails         │   │     │
│   │  │                                                          │   │     │
│   │  │   [Test P2P Connection]  [Configure TURN]              │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  [Load More Results]                                            │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Found: 12 results                                   Categories: Network     │
├─────────────────────────────────────────────────────────────────────────────┤
│ [/] Search  [D] Run Diagnostic  [C] Contact Support  [Esc] Back           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Command Reference

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMMAND REFERENCE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Help Center                                Filter: session_       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Session Management Commands:                                    │     │
│   │                                                                   │     │
│   │  create-session                                                  │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Usage: create-session <name> <total> <threshold>         │   │     │
│   │  │                                                          │   │     │
│   │  │ Description:                                             │   │     │
│   │  │   Creates a new DKG session for wallet generation        │   │     │
│   │  │                                                          │   │     │
│   │  │ Parameters:                                              │   │     │
│   │  │   name      - Unique session identifier                  │   │     │
│   │  │   total     - Total number of participants (2-10)       │   │     │
│   │  │   threshold - Required signatures (1 to total)          │   │     │
│   │  │                                                          │   │     │
│   │  │ Examples:                                                │   │     │
│   │  │   create-session wallet_2of3 3 2                        │   │     │
│   │  │   create-session company-treasury 5 3                   │   │     │
│   │  │                                                          │   │     │
│   │  │ Related: join-session, list-sessions                    │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  join-session                                                    │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Usage: join-session <session-id>                         │   │     │
│   │  │                                                          │   │     │
│   │  │ Description:                                             │   │     │
│   │  │   Join an existing DKG or signing session                │   │     │
│   │  │                                                          │   │     │
│   │  │ Parameters:                                              │   │     │
│   │  │   session-id - The session identifier to join           │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Commands: 24 total                          Showing: Session Management     │
├─────────────────────────────────────────────────────────────────────────────┤
│ [/] Filter  [C] Copy Example  [T] Try in Sandbox  [Esc] Back              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Glossary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GLOSSARY                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Help Center                                    Search: threshold_ │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  MPC Wallet Terminology:                                         │     │
│   │                                                                   │     │
│   │  Threshold Signature                                             │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ A cryptographic signature that requires a minimum number │   │     │
│   │  │ of participants to cooperate. In a 2-of-3 setup, any 2  │   │     │
│   │  │ of the 3 participants can create a valid signature.     │   │     │
│   │  │                                                          │   │     │
│   │  │ Related: FROST, MPC, Key Share                          │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  FROST (Flexible Round-Optimized Schnorr Threshold)             │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ The cryptographic protocol used for threshold signatures │   │     │
│   │  │ in this wallet. FROST enables secure distributed key    │   │     │
│   │  │ generation and signing without any party knowing the    │   │     │
│   │  │ complete private key.                                    │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  DKG (Distributed Key Generation)                                │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ The process where multiple parties jointly generate a    │   │     │
│   │  │ cryptographic key, with each party receiving only a     │   │     │
│   │  │ share. No single party ever sees the complete key.      │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  [A-Z] Browse  [Search]  [Suggest Term]                         │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Terms: 47 total                                     Viewing: T              │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Scroll  [/] Search  [A-Z] Jump to Letter  [Esc] Back                 │
└─────────────────────────────────────────────────────────────────────────────┘
```