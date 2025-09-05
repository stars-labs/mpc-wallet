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
                let mut create_wallet = crate::elm::components::CreateWalletComponent::new();
                
                // Set the selected index from the model
                let selected = self.model.ui_state.selected_indices
                    .get(&crate::elm::model::ComponentId::CreateWallet)
                    .copied()
                    .unwrap_or(0);
                    
                debug!("🔧 Mounting CreateWallet component:");
                debug!("   - Model focus: {:?}", self.model.ui_state.focus);
                debug!("   - Model selected indices: {:?}", self.model.ui_state.selected_indices);
                debug!("   - Setting component selected index to: {}", selected);
                
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
                self.app.mount(
                    Id::CreateWallet,
                    Box::new(crate::elm::components::ModeSelectionComponent::new()),
                    vec![]
                )?;
                self.app.active(&Id::CreateWallet)?;
            }
            Screen::CurveSelection => {
                debug!("🔧 Mounting CurveSelection component");
                self.app.mount(
                    Id::CreateWallet,
                    Box::new(crate::elm::components::CurveSelectionComponent::new()),
                    vec![]
                )?;
                self.app.active(&Id::CreateWallet)?;
            }
            Screen::ThresholdConfig => {
                debug!("🔧 Mounting ThresholdConfig component");
                self.app.mount(
                    Id::CreateWallet,
                    Box::new(crate::elm::components::ThresholdConfigComponent::new()),
                    vec![]
                )?;
                self.app.active(&Id::CreateWallet)?;
            }
            Screen::JoinSession => {
                debug!("🔧 Mounting JoinSession component");
                self.app.mount(
                    Id::CreateWallet,
                    Box::new(crate::elm::components::JoinSessionComponent::new()),
                    vec![]
                )?;
                self.app.active(&Id::CreateWallet)?;
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
        let needs_component_update = matches!(msg, Message::ScrollUp | Message::ScrollDown);
        
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
        let need_remount = self.should_remount() || needs_component_update;
        debug!("Need remount: {} (should_remount: {}, needs_update: {})", 
               need_remount, self.should_remount(), needs_component_update);
        
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
            if let Err(e) = self.mount_components() {
                error!("Failed to mount components: {}", e);
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
                debug!("📺 Received key event: {:?}", key_event);
                
                // Special debug for Esc key at terminal level
                if matches!(key_event.code, crossterm::event::KeyCode::Esc) {
                    debug!("🚨 ESC KEY RECEIVED AT TERMINAL LEVEL!");
                }
                
                let msg = self.handle_key_event(key_event);
                if let Some(msg) = msg {
                    debug!("🎯 Key event produced message: {:?}", msg);
                    self.process_message(msg).await;
                    self.render()?;
                } else {
                    debug!("⚠️ Key event produced no message");
                }
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
        
        // Screen-specific keys
        match key.code {
            KeyCode::Up => {
                debug!("⬆️ Up -> ScrollUp");
                Some(Message::ScrollUp)
            }
            KeyCode::Down => {
                debug!("⬇️ Down -> ScrollDown");
                Some(Message::ScrollDown)
            }
            KeyCode::Enter => {
                // Get the current selected index from the model for the focused component
                let selected_index = self.model.ui_state.selected_indices
                    .get(&self.model.ui_state.focus)
                    .copied()
                    .unwrap_or(0);
                    
                debug!("✅ Enter -> SelectItem with current selected index: {} (focus: {:?})", 
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
                Screen::CreateWallet(_) | Screen::ModeSelection | Screen::CurveSelection | 
                Screen::ThresholdConfig | Screen::JoinSession => {
                    self.app.view(&Id::CreateWallet, f, main_area);
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