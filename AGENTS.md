# AGENTS.md — Feitian SK Manager
## Autonomous Development Roadmap

**Version**: 3.0  
**Last Updated**: November 18, 2025  
**Project Status**: Phase 1 Complete - Ready for Phase 2

---

## 📊 PROGRESS TRACKER

### ✅ Completed Phases

- [x] **Phase 0**: Repository Foundation & Setup
- [x] **Phase 1**: Device Enumeration (HID + CCID)

### 🎯 Current Phase

- [ ] **Phase 2**: Device Connection & Transport Layer (NEXT)

### 📋 Remaining Phases

- [ ] Phase 3: Protocol Detection Implementation
- [ ] Phase 4: FIDO2 Implementation
- [ ] Phase 5: PIV Implementation  
- [ ] Phase 6: OTP Implementation
- [ ] Phase 7-9: U2F, OpenPGP, NDEF
- [ ] Phase 10: Packaging & Distribution
- [ ] Phase 11: Security Hardening
- [ ] Phase 12: Testing & Documentation
- [ ] Phase 13: UI Polish & Animations

---

## 🎯 PROJECT OVERVIEW

### Mission
Build a browser-based Feitian Security Key manager with full support for FIDO2, PIV, OTP, and other protocols using Chrome Extension + Native Messaging Host architecture.

### Architecture
```
Web UI (React + Vite + TypeScript)
        ⇅ window.postMessage
Chrome Extension (Manifest V3)
        ⇅ chrome.runtime.connectNative()
Native Host (Rust - JSON-RPC)
        ⇅ HID (hidapi) + CCID (pcsc)
Feitian Security Key (VID: 0x096e)
```

### Key Constraints
- **Single device operation**: Only one device active at a time
- **Feitian only**: Filter by Vendor ID `0x096e`
- **No code signing**: For development/testing
- **Local testing first**: Publishing is optional (Phase 10)
- **macOS primary**: Cross-platform support secondary

---

## 🛠️ TECHNOLOGY STACK

### Frontend
- React 18+
- Vite (build tool)
- TypeScript (strict mode)
- React Router

### Chrome Extension
- Manifest V3
- Background service worker
- Native messaging

### Native Host
- Rust (latest stable)
- Dependencies:
  - `hidapi` 2.4 - HID device access
  - `pcsc` 2.8 - Smart card access
  - `serde/serde_json` - JSON serialization
  - `tokio` - Async runtime
  - `anyhow` - Error handling

---

## 🎨 DESIGN SYSTEM

### Core Principles
- **Black & White**: High contrast, minimal color
- **Large Radius**: 24px cards, 16px buttons, 12px small elements
- **Smooth Animations**: 300ms page, 150ms interactions
- **Card-Based**: Everything in rounded cards
- **Generous Spacing**: Clean, uncluttered layouts

### Color Palette
```css
--color-bg-primary: #FFFFFF;
--color-bg-secondary: #F5F5F5;
--color-text-primary: #000000;
--color-text-secondary: #666666;
--color-border: #E0E0E0;
--color-success: #10B981;
--color-error: #EF4444;
```

### Typography
```css
--font-family: 'Inter', 'SF Pro', 'Segoe UI', system-ui, sans-serif;
--text-h1: 32px / 600;
--text-h2: 24px / 600;
--text-body: 16px / 400;
```

### Border Radius
```css
--radius-large: 24px;   /* Cards */
--radius-medium: 16px;  /* Buttons */
--radius-small: 12px;   /* Badges */
```

---

## 📁 REPOSITORY STRUCTURE

```
sk-manager-extension/
├── web/                      # React frontend
│   ├── src/
│   │   ├── components/       # Reusable components
│   │   ├── pages/           # Page components
│   │   ├── styles/          # CSS files
│   │   └── App.tsx          # Main app
│   └── package.json
│
├── extension/               # Chrome extension
│   ├── background/service-worker.js
│   ├── content/content.js
│   ├── icons/
│   └── manifest.json
│
├── native/                  # Rust native host
│   ├── src/
│   │   ├── device.rs       # Device enumeration
│   │   ├── protocol.rs     # Protocol detection
│   │   └── main.rs         # JSON-RPC server
│   └── Cargo.toml
│
├── .github/workflows/ci.yml
├── setup-native-host.sh
└── AGENTS.md              # This file
```

---

## 🔄 HOW TO USE THIS FILE (FOR AI AGENTS)

### Step 1: Analyze Repository State
1. Check Progress Tracker above
2. Read "Current State" in next phase
3. Verify prerequisites are met
4. Execute the phase tasks

### Step 2: Execute Phase
1. Read entire phase section
2. Implement all tasks in order
3. Follow design specifications
4. Add comprehensive error handling
5. Write tests where applicable

### Step 3: Verify & Document
1. Test all functionality manually
2. Verify builds pass
3. Update Progress Tracker
4. Mark phase complete

### Step 4: Move to Next Phase
1. Commit all changes
2. Update "Current Phase" marker
3. Read next phase overview
4. Begin implementation

---

## ✅ PHASE 0: Repository Foundation (COMPLETED)

**Status**: ✅ Complete  
**Completion Date**: November 18, 2025

### What Was Built
- Monorepo structure (web, extension, native)
- React + Vite + TypeScript frontend
- Chrome Extension Manifest V3
- Rust native messaging host
- GitHub Actions CI/CD
- Basic JSON-RPC protocol (ping, getVersion)
- Navigation with 5 tabs: Dashboard, FIDO2, PIV, OTP, Protocols

### Verification Checklist
- [x] Web UI builds successfully
- [x] Native host builds successfully
- [x] Extension loads in Chrome without errors
- [x] Extension connects to native host
- [x] Navigation shows 5 tabs only
- [x] CI pipeline passes

---

## ✅ PHASE 1: Device Enumeration (COMPLETED)

**Status**: ✅ Complete  
**Completion Date**: November 18, 2025

### What Was Built
- HID device enumeration (hidapi)
- CCID reader enumeration (pcsc)
- Device filtering by VID 0x096e
- `listDevices` RPC command
- DeviceList UI component
- Dashboard page with device display
- Protocols page structure (placeholder)

### Verification Checklist
- [x] Detects Feitian HID devices
- [x] Detects Feitian CCID readers
- [x] Filters non-Feitian devices
- [x] Dashboard shows device list
- [x] Empty state works ("No devices detected")
- [x] All 11 Rust tests pass

### Files Created
- `/native/src/device.rs` - Device enumeration (293 lines)
- `/native/src/protocol.rs` - Protocol structure (88 lines)
- `/web/src/components/DeviceList.tsx` - Device list component
- `/web/src/pages/Dashboard.tsx` - Dashboard page
- `/web/src/pages/Protocols.tsx` - Protocols page

---

## 🚀 PHASE 2: Device Connection & Transport Layer

**Status**: 🎯 CURRENT PHASE  
**Prerequisites**: Phase 0 & 1 complete

### Overview
Implement device connection management and raw transport layer (HID packets and CCID APDUs). This establishes the foundation for protocol-specific operations.

### Current State Analysis
Before starting, verify:
1. `listDevices` returns Feitian devices successfully
2. Dashboard displays detected devices
3. Native host has `hidapi` and `pcsc` dependencies

---

### TASK 2.1: Add Device Connection State Management

**File**: `/native/src/device.rs`

Add device manager with connection tracking:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct DeviceManager {
    hid_api: Arc<Mutex<hidapi::HidApi>>,
    pcsc_context: Arc<Mutex<pcsc::Context>>,
    open_devices: Arc<Mutex<HashMap<String, OpenDevice>>>,
}

enum OpenDevice {
    Hid(hidapi::HidDevice),
    Ccid(pcsc::Card),
}

impl DeviceManager {
    pub fn new() -> Result<Self, anyhow::Error> {
        let hid_api = hidapi::HidApi::new()?;
        let pcsc_context = pcsc::Context::establish(pcsc::Scope::User)?;
        
        Ok(Self {
            hid_api: Arc::new(Mutex::new(hid_api)),
            pcsc_context: Arc::new(Mutex::new(pcsc_context)),
            open_devices: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub fn open_device(&self, device_id: &str) -> Result<(), anyhow::Error> {
        // Find device in list_devices()
        // Open HID device or CCID card
        // Store in open_devices HashMap
        // Return Ok(()) or error
    }
    
    pub fn close_device(&self, device_id: &str) -> Result<(), anyhow::Error> {
        // Remove from open_devices HashMap
        // Drop closes automatically
    }
    
    pub fn is_open(&self, device_id: &str) -> bool {
        self.open_devices.lock().unwrap().contains_key(device_id)
    }
    
    pub fn get_device(&self, device_id: &str) -> Result<&OpenDevice, anyhow::Error> {
        // Return reference to open device
    }
}
```

**Implementation Requirements**:
1. Thread-safe device management
2. Support both HID and CCID
3. Track open/closed state
4. Error handling for already-open devices
5. Automatic cleanup on drop

---

### TASK 2.2: Implement HID Transport

**File**: `/native/src/transport.rs` (NEW FILE)

```rust
use anyhow::{anyhow, Result};
use hidapi::HidDevice;

/// Send raw HID packet (64 bytes)
pub fn send_hid(device: &HidDevice, data: &[u8]) -> Result<usize> {
    if data.len() > 64 {
        return Err(anyhow!("HID packet too large: {} bytes", data.len()));
    }
    
    let mut padded = vec![0u8; 64];
    padded[..data.len()].copy_from_slice(data);
    
    let bytes_written = device.write(&padded)?;
    log::debug!("Sent HID packet: {} bytes", bytes_written);
    
    Ok(bytes_written)
}

/// Receive raw HID packet
pub fn receive_hid(device: &HidDevice, timeout_ms: i32) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; 64];
    let bytes_read = device.read_timeout(&mut buffer, timeout_ms)?;
    
    if bytes_read == 0 {
        return Err(anyhow!("HID read timeout after {}ms", timeout_ms));
    }
    
    buffer.truncate(bytes_read);
    log::debug!("Received HID packet: {} bytes", bytes_read);
    
    Ok(buffer)
}
```

**Requirements**:
- 64-byte packet size standard
- Timeout support (default 5000ms)
- Automatic padding
- Comprehensive logging
- Error handling

---

### TASK 2.3: Implement CCID/APDU Transport

**Add to**: `/native/src/transport.rs`

```rust
use pcsc::{Card, MAX_BUFFER_SIZE};

/// Transmit APDU to smart card
pub fn transmit_apdu(card: &Card, apdu: &[u8]) -> Result<Vec<u8>> {
    if apdu.len() < 4 {
        return Err(anyhow!("Invalid APDU: too short"));
    }
    
    log::debug!("Transmitting APDU: {} bytes", apdu.len());
    log::trace!("APDU: {:02X?}", apdu);
    
    let mut response = vec![0u8; MAX_BUFFER_SIZE];
    let response_len = card.transmit(apdu, &mut response)?;
    response.truncate(response_len);
    
    // Check status word (last 2 bytes)
    if response.len() < 2 {
        return Err(anyhow!("APDU response too short"));
    }
    
    let sw1 = response[response.len() - 2];
    let sw2 = response[response.len() - 1];
    
    log::debug!("APDU response: {} bytes, SW: {:02X}{:02X}", 
                response.len(), sw1, sw2);
    
    if sw1 != 0x90 || sw2 != 0x00 {
        log::warn!("APDU returned error status: {:02X}{:02X}", sw1, sw2);
    }
    
    Ok(response)
}
```

**Requirements**:
- Support standard APDU format
- Parse status words (SW1 SW2)
- Error code mapping
- Trace-level logging

---

### TASK 2.4: Add RPC Commands

**Update**: `/native/src/main.rs`

Add command handlers:

```rust
"openDevice" => {
    let device_id = params.get("deviceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing deviceId"))?;
    
    device_manager.open_device(device_id)?;
    serde_json::json!({ "success": true, "deviceId": device_id })
}

"closeDevice" => {
    let device_id = params.get("deviceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing deviceId"))?;
    
    device_manager.close_device(device_id)?;
    serde_json::json!({ "success": true, "deviceId": device_id })
}

"sendHid" => {
    let device_id = params.get("deviceId")...;
    let data = params.get("data")...;
    let data_bytes: Vec<u8> = data.iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();
    
    let device = device_manager.get_hid_device(device_id)?;
    let bytes_sent = transport::send_hid(device, &data_bytes)?;
    
    serde_json::json!({ "success": true, "bytesSent": bytes_sent })
}

"receiveHid" => {
    let device_id = params.get("deviceId")...;
    let timeout = params.get("timeout")
        .and_then(|v| v.as_i64())
        .unwrap_or(5000) as i32;
    
    let device = device_manager.get_hid_device(device_id)?;
    let data = transport::receive_hid(device, timeout)?;
    
    serde_json::json!({ "success": true, "data": data })
}

"transmitApdu" => {
    let device_id = params.get("deviceId")...;
    let apdu = params.get("apdu")...;
    let apdu_bytes: Vec<u8> = apdu.iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();
    
    let card = device_manager.get_ccid_card(device_id)?;
    let response = transport::transmit_apdu(card, &apdu_bytes)?;
    
    serde_json::json!({ "success": true, "response": response })
}
```

---

### TASK 2.5: Add Connection UI

**Update**: `/web/src/components/DeviceList.tsx`

Add connect/disconnect functionality:

```typescript
const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null);
const [connecting, setConnecting] = useState<string | null>(null);

const handleConnect = async (deviceId: string) => {
  try {
    setConnecting(deviceId);
    const response = await window.chromeBridge.send('openDevice', { deviceId });
    
    if (response.status === 'ok') {
      setConnectedDeviceId(deviceId);
      toast.success('Device connected');
    }
  } catch (error) {
    toast.error('Connection failed');
  } finally {
    setConnecting(null);
  }
};

const handleDisconnect = async (deviceId: string) => {
  try {
    await window.chromeBridge.send('closeDevice', { deviceId });
    setConnectedDeviceId(null);
    toast.success('Device disconnected');
  } catch (error) {
    toast.error('Disconnect failed');
  }
};

// In device card JSX:
{connectedDeviceId === device.id ? (
  <button onClick={() => handleDisconnect(device.id)}>
    Disconnect
  </button>
) : (
  <button 
    onClick={() => handleConnect(device.id)}
    disabled={connecting === device.id || connectedDeviceId !== null}
  >
    {connecting === device.id ? 'Connecting...' : 'Connect'}
  </button>
)}
```

**UI Requirements**:
- Only one device connected at a time
- Disable other buttons when one connected
- Loading state during connection
- Toast notifications

---

### TASK 2.6: Create Debug Console

**Create**: `/web/src/pages/DebugConsole.tsx`

```typescript
export const DebugConsole: React.FC = () => {
  const [hidData, setHidData] = useState('');
  const [apduData, setApduData] = useState('');
  const [response, setResponse] = useState('');

  const sendHid = async () => {
    // Parse hex string to byte array
    // Send via chromeBridge
    // Display response
  };

  const sendApdu = async () => {
    // Parse hex string to byte array
    // Send via chromeBridge
    // Display response with SW
  };

  return (
    <div className="debug-console">
      <h1>Debug Console</h1>
      
      <div className="card">
        <h2>Send HID Packet</h2>
        <textarea
          value={hidData}
          onChange={(e) => setHidData(e.target.value)}
          placeholder="Enter hex (e.g., 01 02 03 04)"
        />
        <button onClick={sendHid}>Send HID</button>
      </div>

      <div className="card">
        <h2>Send APDU</h2>
        <textarea
          value={apduData}
          onChange={(e) => setApduData(e.target.value)}
          placeholder="Enter APDU hex"
        />
        <button onClick={sendApdu}>Send APDU</button>
      </div>

      <div className="card">
        <h2>Response</h2>
        <pre>{response}</pre>
      </div>
    </div>
  );
};
```

Add to App.tsx routes and Sidebar navigation.

---

### Phase 2 Deliverables

**Code Files**:
- [x] `/native/src/device.rs` - Added DeviceManager
- [x] `/native/src/transport.rs` - NEW (HID + APDU)
- [x] `/native/src/main.rs` - 5 new RPC commands
- [x] `/web/src/components/DeviceList.tsx` - Connect/disconnect
- [x] `/web/src/pages/DebugConsole.tsx` - NEW debug tool

**Functionality**:
- [x] Open/close HID devices
- [x] Open/close CCID cards
- [x] Send/receive raw HID packets
- [x] Transmit APDUs
- [x] Connection state management
- [x] Debug console functional

---

### Phase 2 Testing

1. Build: `cargo build --release`
2. Plug in Feitian device
3. Click "Connect" - should succeed
4. Go to Debug Console
5. Send test HID: `01 02 03 04`
6. Send test APDU: `00 A4 04 00`
7. Click "Disconnect"

Expected: All operations complete without errors.

---

## 🚀 REMAINING PHASES (3-13)

Due to file length, detailed instructions for remaining phases follow the same comprehensive pattern:

### Phase 3: Protocol Detection
- Implement CTAP2, U2F, PIV, OpenPGP, OTP, NDEF detection
- Update Protocols page with real data
- Enable/disable protocol toggles

### Phase 4: FIDO2 Implementation
- PIN management (set, change, retries)
- Credential enumeration and deletion
- Device reset with confirmation
- Complete FIDO2 page UI

### Phase 5: PIV Implementation
- PIN/PUK/Manager Key change
- Certificate management (read, export, import, delete, generate)
- Slot-based operations (9a-9e, f9, 82-95)
- Certificate parsing and display

### Phase 6: OTP Implementation
- 2-slot system (short/long touch)
- Seed input (Base32, Hex, Base64, Plain, CSV)
- Secure seed generator
- Slot swap functionality

### Phases 7-9: U2F, OpenPGP, NDEF
- Similar implementation patterns
- Protocol-specific commands
- UI pages for each

### Phase 10: Packaging & Distribution
- Native host installers (macOS, Windows, Linux)
- Chrome Web Store submission
- Edge Add-ons submission
- Installation documentation

### Phase 11: Security Hardening
- Input validation
- Rate limiting
- Secure memory handling
- Security audit

### Phase 12: Testing & Documentation
- Unit tests
- Integration tests
- E2E tests
- User documentation
- Developer documentation

### Phase 13: UI Polish & Animations
- Page transitions
- Button micro-interactions
- Form animations
- Toast notifications
- Skeleton loading
- Performance optimization

---

## 📋 JSON-RPC PROTOCOL

### Request
```json
{
  "id": 1,
  "command": "commandName",
  "params": { "key": "value" }
}
```

### Response
```json
{
  "id": 1,
  "status": "ok",
  "result": { "data": "..." }
}
```

### Error
```json
{
  "id": 1,
  "status": "error",
  "error": {
    "code": "ERROR_CODE",
    "message": "Description"
  }
}
```

---

## 🐛 TROUBLESHOOTING

### Extension Not Connecting
- Check manifest exists and has correct path
- Verify extension ID matches
- Restart Chrome completely (Cmd+Q)

### Device Not Detected
- Check VID is 0x096e
- Try different USB port
- Close other apps using device

### Build Errors
- Update Rust: `rustup update`
- Clean rebuild: `cargo clean && cargo build`
- For web: `rm -rf node_modules && npm install`

---

## ✅ COMPLETION CHECKLIST

### Completed
- [x] Phase 0: Repository Foundation
- [x] Phase 1: Device Enumeration

### In Progress
- [ ] Phase 2: Device Connection (CURRENT)

### Remaining
- [ ] Phases 3-13

---

**END OF AGENTS.MD**

**Version**: 3.0  
**Last Updated**: November 18, 2025  
**Status**: Phase 1 Complete, Phase 2 Ready
