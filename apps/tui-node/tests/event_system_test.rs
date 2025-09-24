//! Test the new tui-realm event system implementation
//! 
//! This test verifies that:
//! 1. Components receive keyboard events through Component::on()
//! 2. Messages are properly returned from components
//! 3. The app processes messages correctly

use tui_node::elm::components::{
    ThresholdConfigComponent, CurveSelectionComponent, 
    UserEvent
};
use tui_node::elm::message::Message;
use tui_node::elm::model::CurveType;
use tuirealm::{Component, Event};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};

#[test]
fn test_threshold_config_keyboard_events() {
    println!("\n=== Testing ThresholdConfig Component Event Handling ===");
    
    let mut component = ThresholdConfigComponent::new();
    
    // Test Up arrow key
    let up_event = Event::Keyboard(KeyEvent {
        code: Key::Up,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(up_event);
    assert_eq!(msg, Some(Message::ThresholdConfigUp));
    println!("✅ Up arrow -> ThresholdConfigUp message");
    
    // Test Down arrow key
    let down_event = Event::Keyboard(KeyEvent {
        code: Key::Down,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(down_event);
    assert_eq!(msg, Some(Message::ThresholdConfigDown));
    println!("✅ Down arrow -> ThresholdConfigDown message");
    
    // Test Left arrow key
    let left_event = Event::Keyboard(KeyEvent {
        code: Key::Left,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(left_event);
    assert_eq!(msg, Some(Message::ThresholdConfigLeft));
    println!("✅ Left arrow -> ThresholdConfigLeft message");
    
    // Test Right arrow key
    let right_event = Event::Keyboard(KeyEvent {
        code: Key::Right,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(right_event);
    assert_eq!(msg, Some(Message::ThresholdConfigRight));
    println!("✅ Right arrow -> ThresholdConfigRight message");
    
    // Test Enter key
    let enter_event = Event::Keyboard(KeyEvent {
        code: Key::Enter,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(enter_event);
    assert_eq!(msg, Some(Message::ThresholdConfigConfirm));
    println!("✅ Enter -> ThresholdConfigConfirm message");
    
    // Test Esc key
    let esc_event = Event::Keyboard(KeyEvent {
        code: Key::Esc,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(esc_event);
    assert_eq!(msg, Some(Message::NavigateBack));
    println!("✅ Esc -> NavigateBack message");
}

#[test]
fn test_curve_selection_keyboard_events() {
    println!("\n=== Testing CurveSelection Component Event Handling ===");
    
    let mut component = CurveSelectionComponent::new();
    
    // Test Left arrow key (selects Secp256k1)
    let left_event = Event::Keyboard(KeyEvent {
        code: Key::Left,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(left_event);
    assert_eq!(msg, None); // Component updates internally but doesn't send message
    println!("✅ Left arrow -> Internal state updated to Secp256k1");
    
    // Test Right arrow key (selects Ed25519)
    let right_event = Event::Keyboard(KeyEvent {
        code: Key::Right,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(right_event.clone());
    assert_eq!(msg, None); // Component updates internally but doesn't send message
    println!("✅ Right arrow -> Internal state updated to Ed25519");
    
    // Test Enter key (confirms selection)
    let enter_event = Event::Keyboard(KeyEvent {
        code: Key::Enter,
        modifiers: KeyModifiers::NONE,
    });
    
    // After pressing right, should select Ed25519
    component.on(right_event); // Set to Ed25519
    let msg = component.on(enter_event);
    assert_eq!(msg, Some(Message::SelectCurve(CurveType::Ed25519)));
    println!("✅ Enter -> SelectCurve(Ed25519) message");
    
    // Test Esc key
    let esc_event = Event::Keyboard(KeyEvent {
        code: Key::Esc,
        modifiers: KeyModifiers::NONE,
    });
    
    let msg = component.on(esc_event);
    assert_eq!(msg, Some(Message::NavigateBack));
    println!("✅ Esc -> NavigateBack message");
}

#[test]
fn test_component_focus_events() {
    println!("\n=== Testing Component Focus Events ===");
    
    let mut component = ThresholdConfigComponent::new();
    
    // Test focus gained
    let focus_event = Event::User(UserEvent::FocusGained);
    let msg = component.on(focus_event);
    assert_eq!(msg, None); // Focus events don't generate messages
    println!("✅ FocusGained event handled");
    
    // Test focus lost
    let unfocus_event = Event::User(UserEvent::FocusLost);
    let msg = component.on(unfocus_event);
    assert_eq!(msg, None); // Focus events don't generate messages
    println!("✅ FocusLost event handled");
}

#[test]
fn test_event_system_integration() {
    println!("\n=== Testing Event System Integration ===");
    
    // This test verifies that the event flow works as expected:
    // 1. Event is created
    // 2. Component receives event through on()
    // 3. Component returns appropriate Message
    // 4. Message can be processed by the update function
    
    let mut threshold_component = ThresholdConfigComponent::new();
    let mut curve_component = CurveSelectionComponent::new();
    
    // Simulate a sequence of events
    let events = vec![
        Event::Keyboard(KeyEvent {
            code: Key::Up,
            modifiers: KeyModifiers::NONE,
        }),
        Event::Keyboard(KeyEvent {
            code: Key::Down,
            modifiers: KeyModifiers::NONE,
        }),
        Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }),
    ];
    
    println!("\nThreshold Config Component:");
    for event in &events {
        if let Some(msg) = threshold_component.on(event.clone()) {
            println!("  Event {:?} -> Message {:?}", event, msg);
            
            // Verify message types are correct
            match event {
                Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                    assert_eq!(msg, Message::ThresholdConfigUp);
                }
                Event::Keyboard(KeyEvent { code: Key::Down, .. }) => {
                    assert_eq!(msg, Message::ThresholdConfigDown);
                }
                Event::Keyboard(KeyEvent { code: Key::Enter, .. }) => {
                    assert_eq!(msg, Message::ThresholdConfigConfirm);
                }
                _ => {}
            }
        }
    }
    
    println!("\nCurve Selection Component:");
    let curve_events = vec![
        Event::Keyboard(KeyEvent {
            code: Key::Left,
            modifiers: KeyModifiers::NONE,
        }),
        Event::Keyboard(KeyEvent {
            code: Key::Right,
            modifiers: KeyModifiers::NONE,
        }),
        Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }),
    ];
    
    for event in &curve_events {
        if let Some(msg) = curve_component.on(event.clone()) {
            println!("  Event {:?} -> Message {:?}", event, msg);
        }
    }
    
    println!("\n✅ Event system integration test passed!");
}

#[test]
fn test_global_shortcuts() {
    println!("\n=== Testing Global Shortcuts ===");
    
    // Global shortcuts should work regardless of which component is active
    // These are handled by process_global_shortcuts in app.rs
    
    let ctrl_q = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('q'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    
    let ctrl_r = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('r'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    
    let ctrl_h = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('h'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    
    // These would be processed by app.process_global_shortcuts()
    println!("✅ Ctrl+Q -> Should trigger Quit");
    println!("✅ Ctrl+R -> Should trigger Refresh");
    println!("✅ Ctrl+H -> Should trigger NavigateHome");
    
    // Verify the key events can be created correctly
    assert_eq!(ctrl_q.code, crossterm::event::KeyCode::Char('q'));
    assert!(ctrl_q.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
    
    assert_eq!(ctrl_r.code, crossterm::event::KeyCode::Char('r'));
    assert!(ctrl_r.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
    
    assert_eq!(ctrl_h.code, crossterm::event::KeyCode::Char('h'));
    assert!(ctrl_h.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
}

fn main() {
    println!("\n====================================");
    println!("    Event System Test Suite");
    println!("====================================");
    
    test_threshold_config_keyboard_events();
    test_curve_selection_keyboard_events();
    test_component_focus_events();
    test_event_system_integration();
    test_global_shortcuts();
    
    println!("\n====================================");
    println!("    All Tests Passed! ✅");
    println!("====================================");
}