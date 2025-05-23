# Development Guide

## Common Development Issues

### WXT Dev Server Connection Failures

If you encounter "Failed to connect to dev server" errors:

1. **Check Port Availability**
   ```bash
   # Check if port 3000 is in use
   lsof -i :3000
   
   # Kill any processes using the port
   kill -9 <PID>
   ```

2. **Restart Development Server**
   ```bash
   # Stop the dev server
   npm run dev:stop
   
   # Clear any cached builds
   rm -rf .wxt dist
   
   # Restart fresh
   npm run dev
   ```

3. **Alternative Port Configuration**
   ```bash
   # Use a different port
   npm run dev -- --port 3001
   ```

### Offscreen Document Issues

If you see "Receiving end does not exist" errors:

1. **Check Extension State**
   - Open `chrome://extensions/`
   - Click "Inspect views: background page"
   - Look for offscreen document creation logs

2. **Manual Offscreen Creation**
   - In the popup, click "Create Offscreen" button
   - Check background console for creation status

3. **Clear Extension Data**
   ```bash
   # Remove extension and reinstall
   # Or clear all extension data in Chrome settings
   ```

### WebSocket Connection Issues

1. **Check Network Connectivity**
   ```bash
   # Test WebSocket server directly
   wscat -c wss://auto-life.tech
   ```

2. **Firewall/Proxy Issues**
   - Disable corporate firewalls temporarily
   - Check if WebSocket connections are blocked

### Development Workflow

1. **Start Development Server**
   ```bash
   npm run dev
   ```

2. **Load Extension in Chrome**
   - Open `chrome://extensions/`
   - Enable Developer mode
   - Load unpacked extension from `dist/` folder

3. **Monitor Logs**
   - Background script: Inspect views → background page
   - Popup: Right-click popup → Inspect
   - Offscreen: Background console shows offscreen logs

4. **Reload Extension After Changes**
   - Click reload button in `chrome://extensions/`
   - Or use Ctrl+R in background page inspector

### Debugging Tips

1. **Enable Debug Logging**
   - Set `DEBUG=true` in environment
   - Check all console logs in different contexts

2. **Message Flow Debugging**
   - Add breakpoints in message handlers
   - Log message payloads for inspection

3. **State Inspection**
   - Use "Get State" button in popup
   - Check background script variables in debugger

### Performance Considerations

1. **Limit WebRTC Connections**
   - Don't create too many peer connections simultaneously
   - Clean up failed connections promptly

2. **Message Batching**
   - Avoid sending too many messages rapidly
   - Use debouncing for frequent updates

3. **Memory Management**
   - Monitor extension memory usage
   - Clean up event listeners and timers
