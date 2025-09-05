use tui_node::elm::{ElmApp, Model, Message};
use tui_node::utils::appstate_compat::AppState;
use frost_secp256k1::Secp256k1;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_arrow_key_handling() {
    // Initialize app state
    let app_state: Arc<Mutex<AppState<Secp256k1>>> = Arc::new(Mutex::new(
        AppState::new(
            "test-device".to_string(),
            "ws://localhost:9000".to_string(),
            false,
        )
    ));
    
    // Create Elm app
    let elm_app = ElmApp::new("test-device".to_string(), app_state.clone());
    assert!(elm_app.is_ok(), "Failed to create ElmApp");
    
    let mut elm_app = elm_app.unwrap();
    
    // Simulate arrow key events
    let up_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Up,
        crossterm::event::KeyModifiers::NONE,
    );
    
    let down_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Down,
        crossterm::event::KeyModifiers::NONE,
    );
    
    let esc_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
    );
    
    // Test that keys produce appropriate messages
    // Note: We can't directly test handle_key_event as it's private
    // But we can verify that Message::from_global_key works correctly
    
    let esc_msg = Message::from_global_key(esc_key);
    assert_eq!(esc_msg, Some(Message::NavigateBack));
    
    println!("✓ Esc key produces NavigateBack message");
    
    // Test Ctrl+Q
    let quit_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('q'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    
    let quit_msg = Message::from_global_key(quit_key);
    assert_eq!(quit_msg, Some(Message::Quit));
    
    println!("✓ Ctrl+Q produces Quit message");
    
    // Arrow keys shouldn't produce global messages
    let up_msg = Message::from_global_key(up_key);
    assert_eq!(up_msg, None);
    
    let down_msg = Message::from_global_key(down_key);
    assert_eq!(down_msg, None);
    
    println!("✓ Arrow keys don't produce global messages (handled by components)");
}

#[test]
fn test_main_menu_navigation() {
    use tui_node::elm::components::MainMenu;
    
    let mut menu = MainMenu::new();
    
    // Test that we can set selection
    menu.set_selected(0);
    menu.set_selected(3);
    menu.set_selected(5);
    menu.set_selected(10); // Should cap at max
    
    println!("✓ MainMenu selection can be set");
}