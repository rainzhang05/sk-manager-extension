# Chrome Extension

This directory contains the Chrome Extension (Manifest V3) that acts as a bridge between the web UI and the native messaging host.

## Structure

```
extension/
├── manifest.json           # Extension manifest (v3)
├── background/
│   └── service-worker.js  # Background service worker
├── content/
│   └── content.js         # Content script injected into pages
└── icons/                 # Extension icons (to be added)
```

## Features

- **Native Messaging**: Communicates with the Rust native host via JSON-RPC
- **Message Bridge**: Exposes `window.chromeBridge` API to web pages
- **Request Queue**: Manages request/response matching with ID tracking
- **Auto-Reconnect**: Automatically reconnects if the native host disconnects
- **Error Handling**: Comprehensive error handling and validation

## API

The content script exposes `window.chromeBridge` with the following methods:

### `chromeBridge.send(command, params)`
Send a command to the native host.

```javascript
const response = await window.chromeBridge.send('ping', {});
console.log(response); // { status: 'ok', result: { ... } }
```

### `chromeBridge.isConnected()`
Check if connected to the native host.

```javascript
const connected = await window.chromeBridge.isConnected();
console.log(connected); // true or false
```

### `chromeBridge.getVersion()`
Get the native host version.

```javascript
const version = await window.chromeBridge.getVersion();
console.log(version); // "0.1.0"
```

## Loading the Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in top-right)
3. Click "Load unpacked"
4. Select this `extension/` directory
5. The extension should now appear in your extensions list

## Testing

Once loaded, open the browser console on any page and test:

```javascript
// Wait for bridge to be ready
window.addEventListener('chromeBridgeReady', async () => {
  console.log('Bridge ready!');
  
  // Test connection
  const connected = await window.chromeBridge.isConnected();
  console.log('Connected:', connected);
  
  // Test version
  try {
    const version = await window.chromeBridge.getVersion();
    console.log('Native host version:', version);
  } catch (error) {
    console.error('Error:', error);
  }
});
```

## Native Host Configuration

The extension expects a native host named `com.feitian.sk_manager`. The native host manifest must be installed in the correct location:

- **Linux**: `~/.config/google-chrome/NativeMessagingHosts/com.feitian.sk_manager.json`
- **macOS**: `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.feitian.sk_manager.json`
- **Windows**: `HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\com.feitian.sk_manager`

See the `/native` directory for the native host implementation.
