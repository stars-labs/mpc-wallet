#!/usr/bin/env rust-script

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_node::ui::tui::UIMode;
use tui_node::ui::tui_provider::{TuiProvider, TuiState};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Main Menu Navigation Fix");
    
    // Create a test backend
    let backend = TestBackend::new(80, 24);
    let terminal = Terminal::new(backend)?;
    
    // Create TUI provider
    let (tui_provider, _redraw_rx) = TuiProvider::new(terminal);
    
    // Test initial state - should be in MainMenu mode
    let state = tui_provider.get_state_arc();
    {
        let state_guard = state.lock().await;
        if matches!(state_guard.ui_mode, UIMode::MainMenu { selected_index: 0 }) {
            println!("✓ Initial state: MainMenu with selected_index 0");
        } else {
            println!("✗ Initial state incorrect: {:?}", state_guard.ui_mode);
        }
    }
    
    // Test Enter key on first menu item
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let command = tui_provider.handle_key_event(key_event).await;
    
    // Check if UI mode changed to SessionProposalPopup
    {
        let state_guard = state.lock().await;
        if matches!(state_guard.ui_mode, UIMode::SessionProposalPopup { .. }) {
            println!("✓ Enter key correctly transitions to SessionProposalPopup");
        } else {
            println!("✗ Enter key failed to transition. Current mode: {:?}", state_guard.ui_mode);
        }
    }
    
    // Test Escape to go back to main menu
    let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let _command = tui_provider.handle_key_event(esc_event).await;
    
    // Check if UI mode changed back to MainMenu
    {
        let state_guard = state.lock().await;
        if matches!(state_guard.ui_mode, UIMode::MainMenu { selected_index: 0 }) {
            println!("✓ Escape key correctly returns to MainMenu");
        } else {
            println!("✗ Escape key failed to return to MainMenu. Current mode: {:?}", state_guard.ui_mode);
        }
    }
    
    // Test arrow key navigation
    let down_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    let _command = tui_provider.handle_key_event(down_event).await;
    
    {
        let state_guard = state.lock().await;
        if matches!(state_guard.ui_mode, UIMode::MainMenu { selected_index: 1 }) {
            println!("✓ Down arrow correctly increments selected_index to 1");
        } else {
            println!("✗ Down arrow navigation failed. Current mode: {:?}", state_guard.ui_mode);
        }
    }
    
    // Test up arrow navigation
    let up_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    let _command = tui_provider.handle_key_event(up_event).await;
    
    {
        let state_guard = state.lock().await;
        if matches!(state_guard.ui_mode, UIMode::MainMenu { selected_index: 0 }) {
            println!("✓ Up arrow correctly decrements selected_index to 0");
        } else {
            println!("✗ Up arrow navigation failed. Current mode: {:?}", state_guard.ui_mode);
        }
    }
    
    println!("\nTest completed. The Create/Join Session menu item should now work correctly!");
    println!("To test in the actual TUI:");
    println!("1. Run: cargo run");
    println!("2. Press 'm' to open the main menu");
    println!("3. Press Enter on 'Create/Join Session'");
    println!("4. Verify it opens the session proposal form");
    
    Ok(())
}