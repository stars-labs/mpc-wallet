# Offline Mode Wireframes

This document contains wireframes specific to offline mode operations, including QR code displays and manual data exchange screens.

## Offline Mode Indicator

All screens in offline mode show a prominent indicator:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    [🔒 OFFLINE MODE - No Network Connection]                │
├─────────────────────────────────────────────────────────────────────────────┤
```

## Offline Session Creation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CREATE OFFLINE DKG SESSION                               │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                                         [secp256k1]       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Configure offline session:                                      │     │
│   │                                                                   │     │
│   │  Session Name:     [treasury_cold_wallet_______]                │     │
│   │  Total Participants:    [5]                                      │     │
│   │  Threshold:            [3]                                      │     │
│   │                                                                   │     │
│   │  ⚠ Offline Mode Requirements:                                    │     │
│   │  • All participants must be physically present or               │     │
│   │  • Use secure channel for QR code exchange                      │     │
│   │  • Complete all rounds without network connection               │     │
│   │                                                                   │     │
│   │  Your Device ID: mpc-node-001                                   │     │
│   │  Your Participant Index: 1                                      │     │
│   │                                                                   │     │
│   │  [Generate Session Data →]                                       │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Enter] Generate  [Esc] Cancel                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## QR Code Display Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SHARE SESSION DATA                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                     Session: treasury_cold_wallet         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Share this QR code with other participants:                     │     │
│   │                                                                   │     │
│   │                    ┌─────────────────┐                           │     │
│   │                    │ ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄ │                           │     │
│   │                    │ █ ▄▄▄ █▀▄ █ ▄▄▄ █ │                           │     │
│   │                    │ █ ███ █▄▀ █ ███ █ │                           │     │
│   │                    │ █▄▄▄▄▄█ █ █▄▄▄▄▄█ │                           │     │
│   │                    │ ▄▄▄ ▄▄▄▀█▄▄▄ ▄▄▄ │                           │     │
│   │                    │ ▄█▄▀█▄▄ ▀▄█▄▄█▄▀▄ │                           │     │
│   │                    │ █▄▄█▄▄█▀█▄▄▄▀█▄▄█ │                           │     │
│   │                    │ ▄▄▄▄▄▄▄ █▄█ ▄▄▄▄▄ │                           │     │
│   │                    │ █ ▄▄▄ █ ▄  █▄▄█▄▄ │                           │     │
│   │                    │ █ ███ █ ▀▄▄▀▄ ▄▄█ │                           │     │
│   │                    │ █▄▄▄▄▄█ █▀▄▀██▄▄▄ │                           │     │
│   │                    └─────────────────┘                           │     │
│   │                                                                   │     │
│   │  Session ID: TREAS-COLD-2024-0125-1432                          │     │
│   │  Round: 1 of 2                                                   │     │
│   │  Data Size: 2.3 KB                                               │     │
│   │  Checksum: 7A3F...B2E1                                          │     │
│   │                                                                   │     │
│   │  Alternative: [S]ave to File  [C]opy as Text                    │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Participants Ready: 1/5                          Time Elapsed: 00:45        │
├─────────────────────────────────────────────────────────────────────────────┤
│ [N] Next Participant  [R] Regenerate  [S] Save  [Esc] Cancel               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## QR Code Scanner Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SCAN PARTICIPANT DATA                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                     Session: treasury_cold_wallet         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Scan QR code from participant or paste data:                    │     │
│   │                                                                   │     │
│   │  ┌───────────────────────────────────────────────────────────┐  │     │
│   │  │                                                           │  │     │
│   │  │                    [Camera View]                          │  │     │
│   │  │                                                           │  │     │
│   │  │                 ┌─┬─┬─┬─┬─┐                              │  │     │
│   │  │                 ├─┼─┼─┼─┼─┤                              │  │     │
│   │  │                 ├─┼─┼─┼─┼─┤                              │  │     │
│   │  │                 ├─┼─┼─┼─┼─┤                              │  │     │
│   │  │                 └─┴─┴─┴─┴─┘                              │  │     │
│   │  │              Align QR code here                           │  │     │
│   │  │                                                           │  │     │
│   │  │         [M] Manual Input  [F] From File                  │  │     │
│   │  └───────────────────────────────────────────────────────────┘  │     │
│   │                                                                   │     │
│   │  Participants Scanned:                                           │     │
│   │  ✓ Participant 1 (You)                                          │     │
│   │  ✓ Participant 2 - Checksum: 4B2F...A1C3                       │     │
│   │  ⟳ Scanning Participant 3...                                    │     │
│   │  ○ Participant 4 - Waiting                                      │     │
│   │  ○ Participant 5 - Waiting                                      │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Progress: 2/5 participants                       Round: 1/2                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Space] Capture  [M] Manual  [F] File  [N] Skip  [Esc] Cancel             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Manual Data Entry Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MANUAL DATA ENTRY                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                     Session: treasury_cold_wallet         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Enter participant data for Round 1:                             │     │
│   │                                                                   │     │
│   │  Participant Index: [3]                                          │     │
│   │                                                                   │     │
│   │  Data (Base64 or Hex):                                          │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ eyJzZXNzaW9uX2lkIjoidHJlYXN1cnlfY29sZF93YWxsZXQiLCJy  │   │     │
│   │  │ b3VuZCI6MSwicGFydGljaXBhbnRfaW5kZXgiOjMsImRhdGEiOnsic2  │   │     │
│   │  │ hhcmVzIjpbXSwiY29tbWl0bWVudHMiOltdfSwic2lnbmF0dXJlIjoi  │   │     │
│   │  │ MEIwMjIwNzQ4YjgyZDQ0ZmY0YzU5ZjA5ZDQyYzE3YmU5ZTcyMGY5MD  │   │     │
│   │  │ dhMzQ5NDUzNGE3YTU3NjU4NzY1NGY2NTAyMjEwMGE4ZjU0YjQzZmVh  │   │     │
│   │  │ NDU2NzhhMjY3YmQ0NTY3ODkwMTIzNDU2Nzg5MGFiY2RlZl8         │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Format: ● Base64  ○ Hexadecimal                                │     │
│   │                                                                   │     │
│   │  [V] Verify Checksum                                            │     │
│   │                                                                   │     │
│   │  Checksum: 4B2F...A1C3 ✓ Valid                                 │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Data Size: 512 bytes                             Format: Valid Base64       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Enter] Accept  [V] Verify  [C] Clear  [Esc] Cancel                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Offline Signing Request Export

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    EXPORT SIGNING REQUEST                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                          Wallet: treasury_cold_wallet     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Transaction to Sign:                                            │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ To: 0x742d35Cc6634C0532925a3b844Bc9e7595f2bd           │   │     │
│   │  │ Value: 100.5 ETH                                         │   │     │
│   │  │ Gas: 21000 @ 25 Gwei                                     │   │     │
│   │  │ Nonce: 42                                                 │   │     │
│   │  │ Chain: Ethereum Mainnet                                   │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Export Format:                                                 │     │
│   │  ● QR Code Sequence (Recommended for air-gapped)              │     │
│   │  ○ Single File (JSON)                                          │     │
│   │  ○ Multiple Files (One per participant)                        │     │
│   │                                                                   │     │
│   │  Security Options:                                              │     │
│   │  [✓] Add password protection                                   │     │
│   │  [✓] Include verification checksums                            │     │
│   │  [ ] Compress data                                              │     │
│   │                                                                   │     │
│   │  Participants Required: 3 of 5                                  │     │
│   │  Request ID: SIGN-2024-0125-1445                               │     │
│   │  Expires: 24 hours                                              │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Enter] Export  [P] Set Password  [O] Options  [Esc] Cancel                │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Offline Signature Collection

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    COLLECT SIGNATURES                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                          Request: SIGN-2024-0125-1445     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Signature Collection Progress:                                  │     │
│   │                                                                   │     │
│   │  Required: 3 of 5 signatures                                    │     │
│   │  Collected: 2                                                   │     │
│   │                                                                   │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Participant 1 (You)     ✓ Signed at 14:45:23           │   │     │
│   │  │   Signature: 3045022100a8f54b43fea45678a267bd4567... │   │     │
│   │  │                                                         │   │     │
│   │  │ Participant 3           ✓ Signed at 14:47:11           │   │     │
│   │  │   Signature: 304502210098765432109876543210987654... │   │     │
│   │  │                                                         │   │     │
│   │  │ Participant 5           ⟳ Awaiting signature           │   │     │
│   │  │   [Import from: QR Code / File / Manual]               │   │     │
│   │  │                                                         │   │     │
│   │  │ Participant 2           ○ Not required                 │   │     │
│   │  │ Participant 4           ○ Not required                 │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  [I] Import Signature  [V] Verify All  [F] Finalize           │     │
│   │                                                                   │     │
│   │  Time Remaining: 23:12:45                                      │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Status: Need 1 more signature                    Expires: Tomorrow 14:45    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [I] Import  [V] Verify  [F] Finalize  [E] Export Status  [Esc] Cancel     │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Offline Mode Help Screen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    OFFLINE MODE GUIDE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [🔒 OFFLINE MODE]                                                          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Offline Mode Workflow:                                          │     │
│   │                                                                   │     │
│   │  1. DKG Process (All participants together)                      │     │
│   │     • Create session on coordinator device                       │     │
│   │     • Share session QR with all participants                     │     │
│   │     • Each scans others' Round 1 data                           │     │
│   │     • Generate Round 2 data                                      │     │
│   │     • Each scans others' Round 2 data                           │     │
│   │     • Complete DKG and save key shares                          │     │
│   │                                                                   │     │
│   │  2. Signing Process (Can be asynchronous)                        │     │
│   │     • Create signing request on any device                       │     │
│   │     • Export as QR sequence or file                             │     │
│   │     • Share with required participants                           │     │
│   │     • Each participant imports and signs                         │     │
│   │     • Collect signatures back                                    │     │
│   │     • Combine to create final signature                         │     │
│   │                                                                   │     │
│   │  Security Best Practices:                                         │     │
│   │  • Use air-gapped devices for maximum security                  │     │
│   │  • Verify all checksums before accepting data                   │     │
│   │  • Use secure channels for data exchange                        │     │
│   │  • Set expiration times on signing requests                     │     │
│   │  • Password protect sensitive exports                            │     │
│   │                                                                   │     │
│   │  [Page 1/3]                              [→] Next Page           │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [←→] Navigate Pages  [Esc] Close Help                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```