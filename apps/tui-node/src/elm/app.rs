//! ElmApp - Main application using tui-realm with Elm Architecture
//!
//! This is the main application that brings together the Model, Update, View, and Commands
//! to create a fully functional TUI application following the Elm Architecture pattern.

use crate::elm::model::{Model, Screen};
use crate::elm::message::Message;
use crate::elm::update::update;
use crate::elm::components::{Id, MainMenu, WalletList, WalletDetail, ModalComponent, NotificationBar};
use crate::utils::appstate_compat::AppState;

use tuirealm::{Application, EventListenerCfg};
use tuirealm::terminal::TerminalBridge;
use tuirealm::terminal::CrosstermTerminalAdapter;
use ratatui::layout::{Constraint, Direction, Layout};
use crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug, error};

/// The main Elm application
pub struct ElmApp<C: frost_core::Ciphersuite> {
    /// The application model (state)
    model: Model,
    
    /// The tui-realm application
    app: Application<Id, Message, crate::elm::components::UserEvent>,
    
    /// Terminal bridge for rendering
    terminal: TerminalBridge<CrosstermTerminalAdapter>,
    
    /// Channel for sending messages
    message_tx: UnboundedSender<Message>,
    
    /// Channel for receiving messages
    message_rx: UnboundedReceiver<Message>,
    
    /// Reference to the shared app state (for compatibility with existing code)
    app_state: Arc<Mutex<AppState<C>>>,
    
    /// Whether the app should quit
    should_quit: bool,
}

impl<C: frost_core::Ciphersuite + Send + Sync + 'static> ElmApp<C> 
where
    <<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    /// Create a new Elm application
    pub fn new(
        device_id: String,
        app_state: Arc<Mutex<AppState<C>>>,
    ) -> anyhow::Result<Self> {
        // Create message channels
        let (message_tx, message_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Initialize model
        let model = Model::new(device_id);
        
        // Initialize terminal
        let terminal_adapter = CrosstermTerminalAdapter::new()?;
        let terminal = TerminalBridge::new(terminal_adapter);
        
        // Initialize tui-realm application
        let app = Application::init(
            EventListenerCfg::default()
        );
        
        let mut elm_app = Self {
            model,
            app,
            terminal,
            message_tx: message_tx.clone(),
            message_rx,
            app_state,
            should_quit: false,
        };
        
        // Mount initial components
        elm_app.mount_components()?;
        
        // Send initialization message
        let _ = message_tx.send(Message::Initialize);
        
        Ok(elm_app)
    }
    
    /// Mount components based on current screen
    fn mount_components(&mut self) -> anyhow::Result<()> {
        debug!("🔧 Mounting components for screen: {:?}", self.model.current_screen);
        
        // Log state before mounting
        if matches!(self.model.current_screen, Screen::ThresholdConfig) {
            let selected = self.model.ui_state.selected_indices
                .get(&crate::elm::model::ComponentId::ThresholdConfig)
                .copied()
                .unwrap_or(0);
            info!("🔄 PRE-MOUNT: ThresholdConfig selected_field in model = {}", selected);
        }
        
        // Clear all components first
        self.app.umount_all();
        
        // Mount components based on current screen
        match self.model.current_screen {
            Screen::Welcome | Screen::MainMenu => {
                // Create main menu with actual wallet count
                let wallet_count = self.model.wallet_state.wallets.len();
                let mut main_menu = MainMenu::with_wallet_count(wallet_count);
                
                // Set the selected index from the model
                let selected = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::MainMenu)
                    .copied()
                    .unwrap_or(0);
                    
                debug!("Setting MainMenu selected index to: {}, wallet count: {}", selected, wallet_count);
                main_menu.set_selected(selected);
                
                self.app.mount(
                    Id::MainMenu,
                    Box::new(main_menu),
                    vec![]
                )?;
                self.app.active(&Id::MainMenu)?;
            }
            Screen::ManageWallets => {
                let mut wallet_list = WalletList::new();
                wallet_list.set_wallets(self.model.wallet_state.wallets.clone());
                
                self.app.mount(
                    Id::WalletList,
                    Box::new(wallet_list),
                    vec![]
                )?;
                self.app.active(&Id::WalletList)?;
            }
            Screen::WalletDetail { .. } => {
                self.app.mount(
                    Id::WalletDetail,
                    Box::new(WalletDetail::default()),
                    vec![]
                )?;
                self.app.active(&Id::WalletDetail)?;
            }
            Screen::CreateWallet(_) => {
                info!("🔨 Mounting CreateWallet component with state: {:?}", 
                     self.model.wallet_state.creating_wallet);
                
                // Pass the wallet state to the component
                let mut create_wallet = crate::elm::components::CreateWalletComponent::with_state(
                    self.model.wallet_state.creating_wallet.clone()
                );
                
                // Set the selected index from the model
                let selected = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::CreateWallet)
                    .copied()
                    .unwrap_or(0);
                    
                debug!("🔧 Mounting CreateWallet component:");
                debug!("   - Model focus: {:?}", self.model.ui_state.focus);
                debug!("   - Model selected indices: {:?}", self.model.ui_state.selected_indices);
                debug!("   - Setting component selected index to: {}", selected);
                debug!("   - Wallet state: {:?}", self.model.wallet_state.creating_wallet);
                
                create_wallet.set_selected(selected);
                
                self.app.mount(
                    Id::CreateWallet,
                    Box::new(create_wallet),
                    vec![]
                )?;
                self.app.active(&Id::CreateWallet)?;
                
                debug!("✅ CreateWallet component mounted and activated");
            }
            Screen::ModeSelection => {
                debug!("🔧 Mounting ModeSelection component");
                // Get the selected index for ModeSelection
                let selected = self.model.ui_state.selected_indices
                    .get(&self.model.ui_state.focus)
                    .cloned()
                    .unwrap_or(0);
                debug!("ModeSelection selected index: {}", selected);
                self.app.mount(
                    Id::ModeSelection,
                    Box::new(crate::elm::components::ModeSelectionComponent::with_selected(selected)),
                    vec![]
                )?;
                self.app.active(&Id::ModeSelection)?;
            }
            Screen::CurveSelection => {
                debug!("🔧 Mounting CurveSelection component");
                // Get the selected index for CurveSelection
                let selected = self.model.ui_state.selected_indices
                    .get(&self.model.ui_state.focus)
                    .cloned()
                    .unwrap_or(0);
                debug!("CurveSelection selected index: {}", selected);
                self.app.mount(
                    Id::CurveSelection,
                    Box::new(crate::elm::components::CurveSelectionComponent::with_selected(selected)),
                    vec![]
                )?;
                self.app.active(&Id::CurveSelection)?;
            }
            Screen::ThresholdConfig => {
                debug!("🔧 Mounting ThresholdConfig component");
                
                // ALWAYS get the selected field from the correct place
                let selected_field = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::ThresholdConfig)
                    .copied()
                    .unwrap_or(0);
                    
                info!("🎯 ThresholdConfig selected_field from model: {}", selected_field);
                
                // Get values from model if available
                let (participants, threshold) = if let Some(ref creating_wallet) = self.model.wallet_state.creating_wallet {
                    if let Some(ref config) = creating_wallet.custom_config {
                        debug!("Using custom_config values");
                        (config.total_participants, config.threshold)
                    } else {
                        debug!("ThresholdConfig mounting with default values (no custom_config)");
                        (3, 2) // Default values
                    }
                } else {
                    debug!("ThresholdConfig mounting with default values (no creating_wallet)");
                    (3, 2) // Default values
                };
                
                info!("🎯 FINAL: Mounting ThresholdConfig with participants={}, threshold={}, selected_field={}", 
                     participants, threshold, selected_field);
                
                // First unmount if already mounted to force recreation
                if self.app.mounted(&Id::ThresholdConfig) {
                    debug!("Unmounting existing ThresholdConfig component first");
                    let _ = self.app.umount(&Id::ThresholdConfig);
                }
                
                self.app.mount(
                    Id::ThresholdConfig,
                    Box::new(crate::elm::components::ThresholdConfigComponent::with_values(
                        participants, threshold, selected_field
                    )),
                    vec![]
                )?;
                self.app.active(&Id::ThresholdConfig)?;
            }
            Screen::JoinSession => {
                debug!("🔧 Mounting JoinSession component");
                
                // Create component and update it with real sessions from model
                let mut component = crate::elm::components::JoinSessionComponent::new();
                
                // Convert model sessions to UI format
                let ui_sessions: Vec<crate::elm::components::join_session::SessionInfo> = self.model.session_invites
                    .iter()
                    .map(|s| {
                        use crate::elm::components::join_session::{SessionInfo, SessionStatus, SessionType};
                        SessionInfo {
                            id: s.session_id.clone(),
                            session_type: match s.session_type {
                                crate::protocal::signal::SessionType::DKG => SessionType::DKG,
                                crate::protocal::signal::SessionType::Signing { .. } => SessionType::Signing,
                            },
                            creator: s.proposer_id.clone(),
                            status: SessionStatus::Waiting,
                            participants: s.participants.clone(),
                            required: s.total as usize,
                            joined: s.participants.len(),
                            curve: s.curve_type.clone(),
                            mode: s.coordination_type.clone(),
                            created_at: "Just now".to_string(),
                            expires_in: "30 mins".to_string(),
                        }
                    })
                    .collect();
                
                component.update_sessions(ui_sessions);
                
                // Set the selected tab from model
                component.set_selected_tab(self.model.ui_state.join_session_tab);
                debug!("🎯 JoinSession tab set to: {}", if self.model.ui_state.join_session_tab == 0 { "DKG" } else { "Signing" });
                
                // Set the selected index from model
                if let Some(selected_idx) = self.model.ui_state.selected_indices.get(&crate::elm::model::ComponentId::JoinSession) {
                    component.set_selected_index(*selected_idx);
                    debug!("🎯 JoinSession selected index set to: {}", selected_idx);
                }
                
                self.app.mount(
                    Id::JoinSession,
                    Box::new(component),
                    vec![]
                )?;
                self.app.active(&Id::JoinSession)?;
            }
            Screen::DKGProgress { ref session_id } => {
                info!("🔧 Mounting DKGProgress component for session: {}", session_id);
                
                // Get config values from creating_wallet state
                let (total_participants, threshold) = if let Some(ref creating_wallet) = self.model.wallet_state.creating_wallet {
                    if let Some(ref config) = creating_wallet.custom_config {
                        (config.total_participants, config.threshold)
                    } else {
                        (3, 2) // Default values
                    }
                } else {
                    (3, 2) // Default values
                };
                
                // Create the DKG progress component with proper state
                let mut dkg_progress = crate::elm::components::DKGProgressComponent::new(
                    session_id.clone(),
                    total_participants,
                    threshold
                );
                
                // Update WebSocket connection status
                dkg_progress.set_websocket_connected(self.model.network_state.connected);
                
                // Add participants from active session if available
                if let Some(ref session) = self.model.active_session {
                    for participant in &session.participants {
                        dkg_progress.update_participant(
                            participant.clone(),
                            crate::elm::components::dkg_progress::ParticipantStatus::DataChannelOpen
                        );
                    }
                } else {
                    // If no active session, at least add current device
                    dkg_progress.update_participant(
                        self.model.device_id.clone(),
                        crate::elm::components::dkg_progress::ParticipantStatus::DataChannelOpen
                    );
                }
                
                // Set the selected action from the model
                if let Some(selected_action) = self.model.ui_state.selected_indices.get(&crate::elm::model::ComponentId::DKGProgress) {
                    dkg_progress.set_selected_action(*selected_action);
                }
                
                self.app.mount(
                    Id::DKGProgress,
                    Box::new(dkg_progress),
                    vec![]
                )?;
                self.app.active(&Id::DKGProgress)?;
            }
            _ => {
                // Default to main menu for unimplemented screens
                let wallet_count = self.model.wallet_state.wallets.len();
                self.app.mount(
                    Id::MainMenu,
                    Box::new(MainMenu::with_wallet_count(wallet_count)),
                    vec![]
                )?;
                self.app.active(&Id::MainMenu)?;
            }
        }
        
        // Always mount modal and notification components (they control their own visibility)
        self.app.mount(
            Id::Modal,
            Box::new(ModalComponent::default()),
            vec![]
        )?;
        
        self.app.mount(
            Id::NotificationBar,
            Box::new(NotificationBar::default()),
            vec![]
        )?;
        
        Ok(())
    }
    
    /// Process a message through the update function
    async fn process_message(&mut self, msg: Message) {
        info!("📨 Processing message: {:?}", msg);
        
        // Special debug for NavigateBack
        if matches!(msg, Message::NavigateBack) {
            debug!("🚨 PROCESSING NavigateBack MESSAGE!");
        }
        
        // Log the current screen before processing
        debug!("Current screen before: {:?}", self.model.current_screen);
        
        // Check for quit message
        if matches!(msg, Message::Quit) {
            info!("Quit message received, exiting...");
            self.should_quit = true;
            return;
        }
        
        // Check if this is a scroll message that needs component update
        let needs_component_update = matches!(msg, Message::ScrollUp | Message::ScrollDown | Message::ScrollLeft | Message::ScrollRight);
        
        // Check if this is a force remount message
        let force_remount = matches!(msg, Message::ForceRemount);
        if force_remount {
            info!("🔄 ForceRemount detected in app.rs");
        }
        
        // Update the model and get command
        if let Some(command) = update(&mut self.model, msg.clone()) {
            debug!("Update produced command: {:?}", command);
            // Execute the command
            let tx = self.message_tx.clone();
            let app_state = self.app_state.clone();
            
            tokio::spawn(async move {
                if let Err(e) = command.execute(tx, &app_state).await {
                    error!("Command execution failed: {}", e);
                }
            });
        } else {
            debug!("Update produced no command");
        }
        
        // Log the current screen after processing
        debug!("Current screen after: {:?}", self.model.current_screen);
        
        // Check if we need to remount
        let need_remount = self.should_remount() || needs_component_update || force_remount;
        if need_remount {
            info!("🔁 Need remount: {} (should_remount: {}, needs_update: {}, force: {})", 
                   need_remount, self.should_remount(), needs_component_update, force_remount);
        }
        
        // Enhanced debug logging for CreateWallet state sync
        if matches!(self.model.current_screen, Screen::CreateWallet(_)) {
            debug!("🔍 CreateWallet post-update state:");
            debug!("   - Current focus: {:?}", self.model.ui_state.focus);
            debug!("   - Selected indices: {:?}", self.model.ui_state.selected_indices);
            debug!("   - Component mounted: {}", self.app.mounted(&Id::CreateWallet));
            if let Some(selected) = self.model.ui_state.selected_indices.get(&self.model.ui_state.focus) {
                debug!("   - Current selection for focused component: {}", selected);
            }
        }
        
        // Remount components if screen changed or selection updated
        if need_remount {
            debug!("Remounting components for screen: {:?}", self.model.current_screen);
            
            // Add specific debug for ThresholdConfig
            if matches!(self.model.current_screen, Screen::ThresholdConfig) {
                let selected_field = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::ThresholdConfig)
                    .copied()
                    .unwrap_or(0);
                info!("🔄 REMOUNTING ThresholdConfig with selected_field={} from selected_indices", selected_field);
            }
            
            // Add specific debug for DKGProgress
            if matches!(self.model.current_screen, Screen::DKGProgress { .. }) {
                let selected_action = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::DKGProgress)
                    .copied()
                    .unwrap_or(0);
                info!("🔄 REMOUNTING DKGProgress with selected_action={} from selected_indices", selected_action);
            }
            
            if let Err(e) = self.mount_components() {
                error!("Failed to mount components: {}", e);
            }
            
            // Force a render after remounting to ensure UI updates
            if let Err(e) = self.render() {
                error!("Failed to render after remount: {}", e);
            }
        }
        
        // Update component states
        self.update_component_states();
    }
    
    /// Check if components need to be remounted  
    fn should_remount(&self) -> bool {
        // Check if the mounted component matches current screen
        match self.model.current_screen {
            Screen::MainMenu | Screen::Welcome => !self.app.mounted(&Id::MainMenu),
            Screen::ManageWallets => !self.app.mounted(&Id::WalletList),
            Screen::WalletDetail { .. } => !self.app.mounted(&Id::WalletDetail),
            Screen::CreateWallet(_) => !self.app.mounted(&Id::CreateWallet),
            Screen::ModeSelection => !self.app.mounted(&Id::ModeSelection),
            Screen::CurveSelection => !self.app.mounted(&Id::CurveSelection),
            Screen::ThresholdConfig => !self.app.mounted(&Id::ThresholdConfig),
            Screen::JoinSession => !self.app.mounted(&Id::JoinSession),
            Screen::DKGProgress { .. } => !self.app.mounted(&Id::DKGProgress),
            _ => false,
        }
    }
    
    /// Update component states with latest model data
    fn update_component_states(&mut self) {
        // Update MainMenu selection if it's mounted
        if self.app.mounted(&Id::MainMenu) {
            if let Some(selected_idx) = self.model.ui_state.selected_indices.get(&self.model.ui_state.focus) {
                // Unfortunately tuirealm doesn't expose a way to update component state directly
                // We'll need to handle this in the render or via messages
                debug!("Would update MainMenu selected index to: {}", selected_idx);
            }
        }
    }
    
    /// Main event loop
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Starting Elm application event loop");
        
        // Initial render
        self.render()?;
        
        loop {
            // Check if we should quit
            if self.should_quit {
                info!("Quitting application");
                break;
            }
            
            // Poll for events with a small timeout
            tokio::select! {
                // Handle terminal events
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    // Check for crossterm events with a proper timeout
                    if crossterm::event::poll(Duration::from_millis(10))? {
                        match crossterm::event::read() {
                            Ok(event) => {
                                debug!("Read terminal event: {:?}", event);
                                self.handle_terminal_event(event).await?;
                            }
                            Err(e) => {
                                debug!("Error reading terminal event: {:?}", e);
                            }
                        }
                    }
                }
                
                // Handle messages from the update loop
                Some(msg) = self.message_rx.recv() => {
                    self.process_message(msg).await;
                    self.render()?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle terminal events
    async fn handle_terminal_event(&mut self, event: CrosstermEvent) -> anyhow::Result<()> {
        match event {
            CrosstermEvent::Key(key_event) => {
                info!("📺 Received key event: {:?}", key_event);
                
                // Special debug for Enter and Esc keys at terminal level
                if matches!(key_event.code, crossterm::event::KeyCode::Enter) {
                    info!("🔥 ENTER KEY RECEIVED AT TERMINAL LEVEL!");
                }
                if matches!(key_event.code, crossterm::event::KeyCode::Esc) {
                    debug!("🚨 ESC KEY RECEIVED AT TERMINAL LEVEL!");
                }
                
                let msg = self.handle_key_event(key_event);
                if let Some(msg) = msg {
                    debug!("🎯 Key event produced message: {:?}", msg);
                    self.process_message(msg).await;
                }
                // Always render after key events to show component updates
                self.render()?;
            }
            CrosstermEvent::Resize(_, _) => {
                debug!("Terminal resized");
                self.render()?;
            }
            _ => {
                debug!("Other terminal event: {:?}", event);
            }
        }
        
        Ok(())
    }
    
    /// Handle key events - KISS approach, direct crossterm handling
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Option<Message> {
        debug!("🔑 Key pressed: {:?}", key.code);
        
        use crossterm::event::KeyCode;
        
        // Check if modal is open first - modal keys take priority
        if self.model.ui_state.modal.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    debug!("🔙 Modal dismissed with Enter/Esc");
                    return Some(Message::CloseModal);
                }
                _ => return None, // Ignore other keys when modal is open
            }
        }
        
        // Global keys first - work everywhere
        match key.code {
            KeyCode::Esc => {
                debug!("🔙 Esc -> NavigateBack");
                return Some(Message::NavigateBack);
            }
            KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                debug!("🚪 Ctrl+Q -> Quit");
                return Some(Message::Quit);
            }
            KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                debug!("🔄 Ctrl+R -> Refresh");
                return Some(Message::Refresh);
            }
            KeyCode::Char('h') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                debug!("🏠 Ctrl+H -> Home");
                return Some(Message::NavigateHome);
            }
            _ => {}
        }
        
        // For ThresholdConfig screen, we need to update the component's state and then remount
        // Since tuirealm doesn't provide a direct way to send commands to components,
        // we'll update our model and remount the component
        if matches!(self.model.current_screen, Screen::ThresholdConfig) {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    // These are handled by ScrollLeft/ScrollRight which update the model
                    // and trigger a remount
                    if key.code == KeyCode::Left {
                        info!("⬅️ ThresholdConfig LEFT -> ScrollLeft");
                        return Some(Message::ScrollLeft);
                    } else {
                        info!("➡️ ThresholdConfig RIGHT -> ScrollRight");  
                        return Some(Message::ScrollRight);
                    }
                }
                KeyCode::Up | KeyCode::Down => {
                    // For up/down, we need to update the values
                    // Let the normal ScrollUp/ScrollDown handle it
                    if key.code == KeyCode::Up {
                        info!("🔼 ThresholdConfig UP -> ScrollUp");
                        return Some(Message::ScrollUp);
                    } else {
                        info!("🔽 ThresholdConfig DOWN -> ScrollDown");
                        return Some(Message::ScrollDown);
                    }
                }
                KeyCode::Enter => {
                    info!("🔥 ThresholdConfig ENTER -> SelectItem");
                    let selected_index = self.model.ui_state.selected_indices
                        .get(&crate::elm::model::ComponentId::ThresholdConfig)
                        .copied()
                        .unwrap_or(0);
                    return Some(Message::SelectItem { index: selected_index });
                }
                _ => {}
            }
        }
        
        // Screen-specific keys for other screens
        match key.code {
            KeyCode::Up => {
                info!("🔼 UP ARROW KEY PRESSED! -> ScrollUp");
                Some(Message::ScrollUp)
            }
            KeyCode::Down => {
                info!("🔽 DOWN ARROW KEY PRESSED! -> ScrollDown");
                Some(Message::ScrollDown)
            }
            KeyCode::Left => {
                info!("⬅️ LEFT ARROW KEY PRESSED! -> ScrollLeft");
                Some(Message::ScrollLeft)
            }
            KeyCode::Right => {
                info!("➡️ RIGHT ARROW KEY PRESSED! -> ScrollRight");
                Some(Message::ScrollRight)
            }
            KeyCode::Enter => {
                info!("🔥 ENTER KEY PRESSED! Screen: {:?}, Focus: {:?}", 
                     self.model.current_screen, self.model.ui_state.focus);
                
                // Get the current selected index from the model for the focused component
                let selected_index = self.model.ui_state.selected_indices
                    .get(&self.model.ui_state.focus)
                    .copied()
                    .unwrap_or(0);
                    
                info!("✅ Enter -> SelectItem with current selected index: {} (focus: {:?})", 
                       selected_index, self.model.ui_state.focus);
                Some(Message::SelectItem { index: selected_index })
            }
            _ => {
                debug!("❓ Unhandled key: {:?}", key.code);
                None
            }
        }
    }
    
    /// Render the UI
    fn render(&mut self) -> anyhow::Result<()> {
        debug!("🎨 Rendering UI - Current screen: {:?}", self.model.current_screen);
        self.terminal.raw_mut().draw(|f| {
            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(if self.model.ui_state.notifications.is_empty() { 0 } else { 3 }),
                    Constraint::Min(0),
                ])
                .split(f.area());
            
            // Render notification bar if there are notifications
            if !self.model.ui_state.notifications.is_empty() {
                self.app.view(&Id::NotificationBar, f, chunks[0]);
            }
            
            // Render main content based on screen
            let main_area = if self.model.ui_state.notifications.is_empty() {
                f.area()
            } else {
                chunks[1]
            };
            
            // Render active component
            match self.model.current_screen {
                Screen::MainMenu | Screen::Welcome => {
                    self.app.view(&Id::MainMenu, f, main_area);
                }
                Screen::ManageWallets => {
                    self.app.view(&Id::WalletList, f, main_area);
                }
                Screen::WalletDetail { .. } => {
                    self.app.view(&Id::WalletDetail, f, main_area);
                }
                Screen::CreateWallet(_) => {
                    self.app.view(&Id::CreateWallet, f, main_area);
                }
                Screen::ModeSelection => {
                    self.app.view(&Id::ModeSelection, f, main_area);
                }
                Screen::CurveSelection => {
                    self.app.view(&Id::CurveSelection, f, main_area);
                }
                Screen::ThresholdConfig => {
                    self.app.view(&Id::ThresholdConfig, f, main_area);
                }
                Screen::JoinSession => {
                    self.app.view(&Id::JoinSession, f, main_area);
                }
                Screen::DKGProgress { .. } => {
                    self.app.view(&Id::DKGProgress, f, main_area);
                }
                _ => {
                    // Fallback to main menu
                    self.app.view(&Id::MainMenu, f, main_area);
                }
            }
            
            // Render modal if present
            if self.model.ui_state.modal.is_some() {
                // Calculate modal area (centered, smaller than full screen)
                let modal_area = centered_rect(60, 20, main_area);
                self.app.view(&Id::Modal, f, modal_area);
            }
        })?;
        
        Ok(())
    }
    
    /// Get a message sender for external use
    pub fn get_message_sender(&self) -> UnboundedSender<Message> {
        self.message_tx.clone()
    }
}

use std::time::Duration;
use ratatui::layout::Rect;

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Removed unnecessary convert_key_event function - KISS approach!