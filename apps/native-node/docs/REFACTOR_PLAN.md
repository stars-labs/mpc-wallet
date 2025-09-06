# Native Node Refactor Plan

## 📊 Current State Analysis

### Native Node (Current)
The native node currently has:
- ✅ Basic Slint UI with tabs and forms
- ✅ WebSocket connection
- ✅ Simple DKG flow
- ✅ Basic signing functionality
- ✅ Uses TUI node as library (AppRunner)
- ⚠️ Limited feature set compared to TUI
- ⚠️ No offline mode support
- ⚠️ No WebRTC mesh networking
- ⚠️ No keystore session management
- ⚠️ No multi-wallet support
- ⚠️ Limited error handling

### TUI Node (Reference Implementation)
The TUI node has these advanced features:
- ✅ Complete DKG implementation with FROST
- ✅ Offline/Online dual-mode operation
- ✅ WebRTC mesh networking with rejoin
- ✅ Keystore session management
- ✅ Multi-wallet support
- ✅ Import/Export functionality
- ✅ Session discovery and management
- ✅ Advanced signing workflows
- ✅ Comprehensive error handling
- ✅ Audit logging
- ✅ Network partition handling

## 🎯 Missing Components in Native Node

### 1. **Offline Mode Support**
- [ ] SD card import/export UI
- [ ] Offline DKG workflow screens
- [ ] Manual coordination UI
- [ ] QR code generation for data exchange
- [ ] Air-gap status indicators

### 2. **WebRTC Mesh Networking**
- [ ] Peer discovery UI
- [ ] Connection status visualization
- [ ] Mesh topology display
- [ ] Connection quality indicators
- [ ] Rejoin/recovery UI

### 3. **Advanced Keystore Management**
- [ ] Session-based keystore UI
- [ ] Multi-wallet switcher
- [ ] Wallet details view
- [ ] Backup/restore UI
- [ ] Password management

### 4. **Enhanced DKG Features**
- [ ] Visual progress indicators
- [ ] Participant status tracking
- [ ] Round-by-round progress
- [ ] Error recovery UI
- [ ] Threshold configuration

### 5. **Professional Signing UI**
- [ ] Transaction preview
- [ ] Multi-chain support UI
- [ ] Gas estimation
- [ ] Approval workflow
- [ ] Signing history

### 6. **Session Management**
- [ ] Session discovery list
- [ ] Session details view
- [ ] Participant management
- [ ] Session state visualization
- [ ] Rejoin capabilities

## 🏗️ Refactor Architecture

### Phase 1: UI Component Library
Create reusable Slint components matching TUI features:

```
ui/
├── components/
│   ├── connection_status.slint    # WebSocket/WebRTC status
│   ├── wallet_selector.slint      # Multi-wallet dropdown
│   ├── session_list.slint         # Session discovery
│   ├── participant_list.slint     # DKG/signing participants
│   ├── progress_indicator.slint   # Multi-step progress
│   ├── offline_mode.slint         # Offline mode controls
│   └── mesh_topology.slint        # WebRTC mesh visualization
├── dialogs/
│   ├── create_wallet.slint        # Wallet creation wizard
│   ├── import_export.slint        # Import/export dialog
│   ├── signing_approval.slint     # Transaction approval
│   └── error_recovery.slint       # Error handling dialog
└── screens/
    ├── dashboard.slint             # Main dashboard
    ├── wallet_management.slint     # Wallet operations
    ├── session_management.slint    # Session operations
    ├── offline_operations.slint    # Offline mode screen
    └── settings.slint              # Configuration
```

### Phase 2: State Management
Implement comprehensive state management:

```rust
// Enhanced AppState global
export global AppState {
    // Connection State
    in-out property <bool> websocket_connected;
    in-out property <bool> webrtc_connected;
    in-out property <[PeerConnection]> mesh_connections;
    
    // Wallet State
    in-out property <[Wallet]> wallets;
    in-out property <int> active_wallet_index;
    in-out property <bool> has_keystore;
    
    // Session State
    in-out property <[Session]> available_sessions;
    in-out property <Session> active_session;
    in-out property <SessionPhase> current_phase;
    
    // Mode State
    in-out property <OperationMode> mode; // Online/Offline
    in-out property <bool> sd_card_present;
    
    // DKG State
    in-out property <DkgProgress> dkg_progress;
    in-out property <[Participant]> participants;
    
    // Signing State
    in-out property <[SigningRequest]> pending_requests;
    in-out property <Transaction> current_transaction;
}
```

### Phase 3: Feature Implementation Priority

#### High Priority (Week 1-2)
1. **Keystore Session Management**
   - Implement session-based keystore UI
   - Add wallet switching capability
   - Import/export functionality

2. **Enhanced DKG UI**
   - Visual progress tracking
   - Participant status display
   - Error recovery flows

3. **Professional Signing Workflow**
   - Transaction preview screen
   - Approval workflow
   - History tracking

#### Medium Priority (Week 3-4)
4. **Offline Mode Support**
   - SD card operations UI
   - Manual coordination screens
   - Offline indicators

5. **WebRTC Mesh Visualization**
   - Connection status display
   - Mesh topology view
   - Quality indicators

6. **Session Discovery**
   - Available sessions list
   - Join/create workflows
   - Session details

#### Low Priority (Week 5-6)
7. **Advanced Features**
   - Multi-chain UI
   - QR code support
   - Audit log viewer
   - Settings management

## 🔧 Implementation Strategy

### Step 1: Create UI Components
```rust
// Example: WalletSelector component
export component WalletSelector {
    in property <[Wallet]> wallets;
    in-out property <int> selected_index;
    callback wallet_changed(int);
    
    ComboBox {
        model: wallets;
        current-index: selected_index;
        current-value: wallets[selected_index].name;
        selected(index) => {
            selected_index = index;
            wallet_changed(index);
        }
    }
}
```

### Step 2: Extend UIProvider Implementation
```rust
impl UIProvider for NativeUIProvider {
    // Add missing methods
    async fn show_offline_mode_dialog(&self) { ... }
    async fn update_mesh_topology(&self, peers: Vec<PeerInfo>) { ... }
    async fn show_session_discovery(&self, sessions: Vec<SessionInfo>) { ... }
    async fn update_dkg_progress(&self, round: u8, progress: f32) { ... }
    async fn show_signing_approval(&self, tx: Transaction) -> bool { ... }
}
```

### Step 3: Integrate TUI Features
```rust
// Use existing TUI handlers
use tui_node::handlers::{
    keystore_commands,
    offline_commands,
    session_handler,
    signing_commands,
    wallet_commands,
};

// Adapt for native UI
impl NativeApp {
    async fn handle_keystore_command(&mut self, cmd: KeystoreCommand) {
        match cmd {
            KeystoreCommand::CreateSession { .. } => {
                // Update UI state
                self.ui_provider.show_session_creation_dialog().await;
                // Call TUI handler
                keystore_commands::handle_create_session(...).await;
            }
            // ... other commands
        }
    }
}
```

## 📋 Testing Requirements

### UI Testing
- [ ] Component unit tests
- [ ] Integration tests with TUI backend
- [ ] User flow E2E tests
- [ ] Offline mode simulation
- [ ] WebRTC mesh simulation

### Feature Testing
- [ ] Keystore operations
- [ ] DKG with disconnections
- [ ] Signing workflows
- [ ] Session management
- [ ] Import/export

## 🎨 UI/UX Improvements

### Visual Enhancements
- Use consistent color scheme
- Add animations for state transitions
- Implement dark/light theme
- Add loading states
- Show tooltips and help text

### Navigation
- Implement breadcrumb navigation
- Add keyboard shortcuts
- Support tab navigation
- Add context menus

### Accessibility
- Ensure screen reader compatibility
- Add high contrast mode
- Support keyboard-only navigation
- Provide audio feedback

## 📊 Success Metrics

### Functionality
- ✅ Feature parity with TUI node
- ✅ All E2E tests passing
- ✅ Offline mode fully functional
- ✅ WebRTC mesh working

### Performance
- UI responsiveness < 100ms
- Smooth animations (60 FPS)
- Memory usage < 200MB
- CPU usage < 10% idle

### User Experience
- Intuitive navigation
- Clear error messages
- Visual feedback for all actions
- Consistent behavior

## 🚀 Migration Path

### Phase 1: Foundation (Week 1)
1. Set up component library structure
2. Create base components
3. Implement enhanced AppState
4. Update UIProvider interface

### Phase 2: Core Features (Week 2-3)
1. Implement keystore UI
2. Add DKG visualization
3. Create signing workflow
4. Add session management

### Phase 3: Advanced Features (Week 4-5)
1. Add offline mode UI
2. Implement WebRTC visualization
3. Add import/export
4. Create settings screen

### Phase 4: Polish (Week 6)
1. Add animations
2. Implement themes
3. Add help system
4. Performance optimization

## 🔄 Backwards Compatibility

- Maintain existing AppRunner interface
- Keep current UI functional during migration
- Support gradual feature rollout
- Preserve existing keystore format

## 📚 Documentation Updates

- [ ] Update native node README
- [ ] Create UI component guide
- [ ] Document new workflows
- [ ] Add screenshot examples
- [ ] Create video tutorials

## 🎯 End Goal

Transform the native node from a basic UI into a professional-grade MPC wallet application that:
- Matches all TUI node capabilities
- Provides superior visual experience
- Supports enterprise requirements
- Maintains high security standards
- Offers intuitive user experience

This refactor will position the native node as the premier desktop MPC wallet solution, combining the power of the TUI backend with a modern, accessible GUI.