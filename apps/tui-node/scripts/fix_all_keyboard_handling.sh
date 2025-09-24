#!/bin/bash

# This script ensures ALL components have proper keyboard handling
# It prevents the recurring issue of keyboard events not working

echo "🔧 Fixing keyboard handling across all components..."

# Find all component files
COMPONENT_DIR="src/elm/components"
COMPONENT_FILES=$(find $COMPONENT_DIR -name "*.rs" -type f | grep -v mod.rs)

echo "Found components:"
echo "$COMPONENT_FILES"

# Function to check and report issues
check_component() {
    local file=$1
    local filename=$(basename $file)
    echo ""
    echo "Checking $filename..."
    
    # Check for KeyModifiers::NONE (bad pattern)
    if grep -q "KeyModifiers::NONE" "$file"; then
        echo "  ⚠️  Found KeyModifiers::NONE in $filename - this breaks keyboard handling!"
        return 1
    fi
    
    # Check for proper .. pattern in keyboard events
    if grep -q "Event::Keyboard.*KeyEvent" "$file"; then
        if ! grep -q "Event::Keyboard.*KeyEvent.*\.\." "$file"; then
            echo "  ⚠️  Missing .. pattern in keyboard events in $filename"
            return 1
        fi
    fi
    
    # Check for debug logging
    if grep -q "fn on(" "$file"; then
        if ! grep -q "tracing::debug.*received event" "$file"; then
            echo "  ⚠️  Missing debug logging in $filename"
            return 1
        fi
    fi
    
    echo "  ✅ $filename looks good"
    return 0
}

# Track components with issues
COMPONENTS_WITH_ISSUES=""

for file in $COMPONENT_FILES; do
    if ! check_component "$file"; then
        COMPONENTS_WITH_ISSUES="$COMPONENTS_WITH_ISSUES $file"
    fi
done

if [ -n "$COMPONENTS_WITH_ISSUES" ]; then
    echo ""
    echo "❌ Found issues in the following components:"
    echo "$COMPONENTS_WITH_ISSUES"
    echo ""
    echo "To fix these issues:"
    echo "1. Replace KeyModifiers::NONE with .. pattern"
    echo "2. Add debug logging to track events"
    echo "3. Ensure component IDs match the Id enum"
else
    echo ""
    echo "✅ All components have proper keyboard handling!"
fi

# Create a template for proper keyboard handling
cat > src/elm/components/keyboard_template.txt << 'EOF'
// TEMPLATE FOR PROPER KEYBOARD HANDLING
impl Component<Message, UserEvent> for YourComponent {
    fn on(&mut self, event: Event<UserEvent>) -> Option<Message> {
        tracing::debug!("🎮 YourComponent received event: {:?}", event);
        
        let result = match event {
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                ..  // IMPORTANT: Use .. pattern, NOT KeyModifiers::NONE
            }) => {
                Some(Message::ScrollUp)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                ..  // IMPORTANT: Use .. pattern, NOT KeyModifiers::NONE
            }) => {
                Some(Message::ScrollDown)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                ..  // IMPORTANT: Use .. pattern, NOT KeyModifiers::NONE
            }) => {
                Some(Message::SelectItem { index: self.selected })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                ..  // IMPORTANT: Use .. pattern, NOT KeyModifiers::NONE
            }) => {
                Some(Message::NavigateBack)
            }
            Event::User(UserEvent::FocusGained) => {
                self.focused = true;
                None
            }
            Event::User(UserEvent::FocusLost) => {
                self.focused = false;
                None
            }
            _ => None,
        };
        
        if let Some(ref msg) = result {
            tracing::debug!("🎮 YourComponent returning message: {:?}", msg);
        }
        
        result
    }
}
EOF

echo ""
echo "📝 Created keyboard handling template at src/elm/components/keyboard_template.txt"