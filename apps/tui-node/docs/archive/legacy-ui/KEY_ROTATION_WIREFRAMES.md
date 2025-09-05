# Key Rotation Path Wireframes

This document contains detailed wireframes for the Key Rotation functionality, a critical security feature for enterprise MPC wallet operations.

## Table of Contents

1. [Key Rotation Menu](#key-rotation-menu)
2. [Rotation Configuration](#rotation-configuration)
3. [Participant Management](#participant-management)
4. [Rotation Progress](#rotation-progress)
5. [Verification & Completion](#verification--completion)

---

## Key Rotation Menu

Main screen for key rotation operations.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KEY ROTATION                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Recovery Menu                    Last Rotation: 45 days ago      │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Key Rotation Options:                                           │     │
│   │                                                                   │     │
│   │  > [1] Standard Rotation                                         │     │
│   │      Rotate all key shares (recommended quarterly)               │     │
│   │                                                                   │     │
│   │    [2] Emergency Rotation                                        │     │
│   │      Immediate rotation due to security concerns                 │     │
│   │                                                                   │     │
│   │    [3] Add Participants                                          │     │
│   │      Increase the number of key holders                         │     │
│   │                                                                   │     │
│   │    [4] Remove Participants                                       │     │
│   │      Revoke access for specific participants                    │     │
│   │                                                                   │     │
│   │    [5] Change Threshold                                          │     │
│   │      Modify signature requirements (e.g., 2/3 to 3/5)          │     │
│   │                                                                   │     │
│   │    [6] Rotation History                                          │     │
│   │      View past rotation events and audit logs                   │     │
│   │                                                                   │     │
│   │  Current Configuration:                                          │     │
│   │  • Threshold: 2 of 3                                            │     │
│   │  • Active Participants: 3                                       │     │
│   │  • Rotation Policy: Quarterly (next due: 15 days)              │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Security Status: ✓ Secure                  Compliance: ✓ Up to date        │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓/1-6] Navigate  [Enter] Select  [P] View Policy  [Esc] Back            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Rotation Configuration

Configure parameters for key rotation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CONFIGURE KEY ROTATION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Key Rotation Menu               Type: Standard Rotation          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Rotation Parameters:                                            │     │
│   │                                                                   │     │
│   │  Selected Wallet: company_treasury                               │     │
│   │  Current Setup: 2-of-3 multisig                                 │     │
│   │                                                                   │     │
│   │  New Configuration:                                              │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Total Participants:  [3] → [3]  (no change)             │   │     │
│   │  │ Threshold:          [2] → [2]  (no change)             │   │     │
│   │  │                                                           │   │     │
│   │  │ Rotation Type:                                           │   │     │
│   │  │ [⚫] Full Rotation - All shares regenerated             │   │     │
│   │  │ [○] Partial Rotation - Selected shares only            │   │     │
│   │  │                                                           │   │     │
│   │  │ Timing:                                                 │   │     │
│   │  │ [⚫] Start Immediately                                   │   │     │
│   │  │ [○] Schedule for: [____-__-__ __:__]                   │   │     │
│   │  │                                                           │   │     │
│   │  │ Notification Settings:                                   │   │     │
│   │  │ [✓] Email all participants                             │   │     │
│   │  │ [✓] SMS critical alerts                                │   │     │
│   │  │ [✓] In-app notifications                               │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Security Requirements:                                          │     │
│   │  • All participants must be online during rotation              │     │
│   │  • Process cannot be interrupted once started                   │     │
│   │  • Estimated duration: 15-20 minutes                            │     │
│   │                                                                   │     │
│   │  [✓] I understand the implications of key rotation              │     │
│   │  [✓] All participants have been notified                       │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Validation: ✓ Ready to proceed              [Preview] [Start Rotation]     │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Tab] Next Field  [Space] Toggle  [P] Preview  [S] Start  [Esc] Cancel     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Participant Management

Screen for adding or removing participants during rotation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MANAGE PARTICIPANTS                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ « Back to Rotation Config                  Changes: Adding 2, Removing 1   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  Current Participants:                                           │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ [✓] mpc-node-001    Role: Admin      Status: ● Active   │   │     │
│   │  │     Joined: 2023-12-01   Last active: 2 minutes ago    │   │     │
│   │  │                                                           │   │     │
│   │  │ [✓] mpc-node-002    Role: Signer     Status: ● Active   │   │     │
│   │  │     Joined: 2023-12-01   Last active: 1 hour ago       │   │     │
│   │  │                                                           │   │     │
│   │  │ [✗] mpc-node-003    Role: Signer     Status: ○ Remove   │   │     │
│   │  │     Joined: 2023-12-15   Reason: Employee departure    │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  New Participants:                                              │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ [+] mpc-node-004    Role: Signer     Status: ◐ Pending  │   │     │
│   │  │     Device ID: [node-004-prod_______________]           │   │     │
│   │  │     Public Key: [Awaiting device registration]          │   │     │
│   │  │                                                           │   │     │
│   │  │ [+] mpc-node-005    Role: Backup     Status: ◐ Pending  │   │     │
│   │  │     Device ID: [node-005-backup_____________]           │   │     │
│   │  │     Public Key: [Awaiting device registration]          │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  New Threshold Configuration:                                   │     │
│   │  Current: 2 of 3 → New: 3 of 4                                 │     │
│   │                                                                   │     │
│   │  [A] Add Another  [R] Remove Selected  [T] Test Configuration  │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Total after rotation: 4 participants        Required signatures: 3         │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Space] Toggle  [A] Add  [R] Remove  [C] Continue           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Rotation Progress

Real-time progress tracking during key rotation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KEY ROTATION IN PROGRESS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Wallet: company_treasury                          Started: 14:32:00        │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  ⚠ DO NOT INTERRUPT THIS PROCESS                                │     │
│   │                                                                   │     │
│   │  Rotation Progress:                                              │     │
│   │                                                                   │     │
│   │  Phase 1: Initialization                                         │     │
│   │  [████████████████████████████████████████] 100% Complete       │     │
│   │                                                                   │     │
│   │  Phase 2: Participant Verification                               │     │
│   │  [████████████████████████████████████████] 100% Complete       │     │
│   │                                                                   │     │
│   │  Phase 3: New Key Generation (FROST DKG)                         │     │
│   │  [██████████████████████░░░░░░░░░░░░░░░░░░] 60% In Progress     │     │
│   │                                                                   │     │
│   │  Phase 4: Share Distribution                                      │     │
│   │  [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0% Pending          │     │
│   │                                                                   │     │
│   │  Phase 5: Verification & Cleanup                                 │     │
│   │  [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0% Pending          │     │
│   │                                                                   │     │
│   │  Participant Status:                                             │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ • mpc-node-001    ✓ Connected   ⟳ Generating shares    │   │     │
│   │  │ • mpc-node-002    ✓ Connected   ✓ Shares generated     │   │     │
│   │  │ • mpc-node-004    ✓ Connected   ⟳ Generating shares    │   │     │
│   │  │ • mpc-node-005    ✓ Connected   ⧖ Waiting              │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Messages: 127 exchanged    Data: 45.2 KB transferred          │     │
│   │  Estimated time remaining: ~8 minutes                           │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Elapsed: 00:07:32                          Network: ● Stable              │
├─────────────────────────────────────────────────────────────────────────────┤
│ [L] View Logs  [P] Pause  [?] Help                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Verification & Completion

Final verification and completion screen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ROTATION COMPLETE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Wallet: company_treasury                   Duration: 15:42                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  ✓ Key Rotation Successfully Completed                          │     │
│   │                                                                   │     │
│   │  Summary:                                                        │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ Previous Configuration:  2 of 3 multisig                │   │     │
│   │  │ New Configuration:       3 of 4 multisig                │   │     │
│   │  │ Participants Updated:    +2 added, -1 removed           │   │     │
│   │  │ New Wallet Address:      0x9F2E4...7C3A (unchanged)     │   │     │
│   │  │ Rotation Type:          Full rotation                   │   │     │
│   │  │ Completion Time:        2024-01-25 14:47:42 UTC        │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Verification Results:                                           │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ ✓ All shares verified and functional                    │   │     │
│   │  │ ✓ Threshold signatures tested successfully              │   │     │
│   │  │ ✓ Old shares securely destroyed                         │   │     │
│   │  │ ✓ Backup created and encrypted                          │   │     │
│   │  │ ✓ Audit log updated                                     │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  New Participant Keys:                                           │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ mpc-node-001: Share 1 of 4 [Verified]                  │   │     │
│   │  │ mpc-node-002: Share 2 of 4 [Verified]                  │   │     │
│   │  │ mpc-node-004: Share 3 of 4 [Verified]                  │   │     │
│   │  │ mpc-node-005: Share 4 of 4 [Verified]                  │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Next Steps:                                                     │     │
│   │  • Test new configuration with a small transaction              │     │
│   │  • Update access control policies                               │     │
│   │  • Schedule next rotation (90 days)                             │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ [E] Export Report  [T] Test Transaction  [B] Create Backup  [D] Done       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [E] Export  [T] Test  [B] Backup  [D] Done                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Emergency Rotation Screen

Special flow for emergency key rotation scenarios.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      🚨 EMERGENCY KEY ROTATION 🚨                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Reason: Security Breach Detected            Initiated: 14:55:00            │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │  ⚠ CRITICAL SECURITY OPERATION IN PROGRESS                      │     │
│   │                                                                   │     │
│   │  Threat Assessment:                                              │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ • Compromised Device: mpc-node-003                       │   │     │
│   │  │ • Risk Level: HIGH                                        │   │     │
│   │  │ • Affected Wallets: 3                                     │   │     │
│   │  │ • Immediate Action: REQUIRED                              │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   │  Emergency Protocol:                                             │     │
│   │  [████████████████████████████░░░░░░░░░] 75% In Progress       │     │
│   │                                                                   │     │
│   │  Actions Completed:                                              │     │
│   │  ✓ All wallets locked                                           │     │
│   │  ✓ Compromised participant isolated                             │     │
│   │  ✓ Emergency contacts notified                                  │     │
│   │  ⟳ Generating new key shares...                                 │     │
│   │  ○ Pending: Distribute new shares                               │     │
│   │  ○ Pending: Verify and activate                                 │     │
│   │                                                                   │     │
│   │  Available Participants:                                         │     │
│   │  ┌─────────────────────────────────────────────────────────┐   │     │
│   │  │ mpc-node-001: ● Online   ✓ Verified   ⟳ Rotating       │   │     │
│   │  │ mpc-node-002: ● Online   ✓ Verified   ✓ Complete       │   │     │
│   │  │ mpc-node-003: ✗ BLOCKED  ⚠ Compromised                 │   │     │
│   │  │ mpc-backup-1: ● Online   ✓ Activated  ⟳ Rotating       │   │     │
│   │  └─────────────────────────────────────────────────────────┘   │     │
│   │                                                                   │     │
│   └─────────────────────────────────────────────────────────────────┘     │
│                                                                             │
│ Time Elapsed: 00:03:27                     Est. Completion: 00:02:00       │
├─────────────────────────────────────────────────────────────────────────────┤
│ [A] Abort (NOT RECOMMENDED)  [L] View Logs  [?] Emergency Help             │
└─────────────────────────────────────────────────────────────────────────────┘
```