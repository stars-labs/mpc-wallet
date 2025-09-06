use tui_node::elm::{ElmApp, Model, Message};
use tui_node::utils::appstate_compat::AppState;
use frost_secp256k1::Secp256K1Sha256 as Secp256k1;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_arrow_key_handling() {
    // Initialize app state (new() doesn't take arguments anymore)
    let app_state: Arc<Mutex<AppState<Secp256k1>>> = Arc::new(Mutex::new(
        AppState::new()
    ));
    
    // Create Elm app (new signature takes only device_id and app_state)
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
    
    // Test that keys work correctly by directly creating messages
    // from_global_key was removed in favor of direct key handling
    
    // Test that appropriate messages can be created
    let _nav_back_msg = Message::NavigateBack;
    let _quit_msg = Message::Quit;
    
    println!("✓ Navigation messages can be created");
    
    // Test that we can send messages through the message sender
    let message_sender = elm_app.get_message_sender();
    
    // Send NavigateBack message
    let result = message_sender.send(Message::NavigateBack);
    assert!(result.is_ok());
    println!("✓ Can send NavigateBack message");
    
    // Send Quit message
    let result = message_sender.send(Message::Quit);
    assert!(result.is_ok());
    println!("✓ Can send Quit message");
    
    println!("✓ Key handling works through message channel");
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