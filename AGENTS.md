# AGENTS.md — Feitian SK Manager
## Autonomous Development Roadmap

**Version**: 3.1  
**Last Updated**: November 18, 2025  
**Project Status**: Phase 2 Complete - Ready for Phase 3

---

## 📊 PROGRESS TRACKER

### ✅ Completed Phases

- [x] **Phase 0**: Repository Foundation & Setup
- [x] **Phase 1**: Device Enumeration (HID + CCID)
- [x] **Phase 2**: Device Connection & Transport Layer

### 🎯 Current Phase

- [ ] **Phase 3**: Protocol Detection Implementation (NEXT)

### 📋 Remaining Phases

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

## ✅ PHASE 2: Device Connection & Transport Layer (COMPLETED)

**Status**: ✅ Complete  
**Completion Date**: November 18, 2025

### Overview
Implemented device connection management and raw transport layer (HID packets and CCID APDUs). This establishes the foundation for protocol-specific operations.

### What Was Built
- DeviceManager for thread-safe connection tracking
- HID transport (send/receive 64-byte packets)
- CCID/APDU transport with status word parsing
- 5 new RPC commands (openDevice, closeDevice, sendHid, receiveHid, transmitApdu)
- Connect/disconnect UI in DeviceList component
- Debug Console page for manual testing
- Single-device connection enforcement

### Verification Checklist
- [x] DeviceManager creates and manages HID/CCID connections
- [x] HID send/receive operations work correctly
- [x] APDU transmit operation works correctly
- [x] RPC commands handle errors gracefully
- [x] UI allows connecting/disconnecting devices
- [x] Only one device can be connected at a time
- [x] Debug Console provides hex input/output
- [x] All 15 tests pass (11 original + 4 new transport tests)
- [x] Web UI builds without errors
- [x] Native host builds without errors

### Files Created/Modified
**Native Host:**
- `/native/src/device.rs` - Added DeviceManager (190 lines added)
- `/native/src/transport.rs` - NEW file (170 lines)
- `/native/src/main.rs` - Added 5 RPC handlers (200+ lines added)

**Frontend:**
- `/web/src/components/DeviceList.tsx` - Added connection logic (100+ lines modified)
- `/web/src/styles/DeviceList.css` - Added connected state styles
- `/web/src/pages/DebugConsole.tsx` - NEW debug console (280 lines)
- `/web/src/styles/DebugConsole.css` - NEW styling (150 lines)
- `/web/src/pages/index.ts` - Added DebugConsole export
- `/web/src/App.tsx` - Added /debug route

---

## 🚀 PHASE 3: Protocol Detection Implementation

**Status**: 🎯 CURRENT PHASE  
**Prerequisites**: Phase 0, 1 & 2 complete

### Overview
Implement protocol detection for CTAP2, U2F, PIV, OpenPGP, OTP, and NDEF. Update Protocols page to display real detection results instead of placeholder data.

### Current State Analysis
Phase 2 implementation is complete. All device connection and transport functionality is working.

---

## 🚀 PHASE 3: Protocol Detection Implementation

**Status**: 🎯 CURRENT PHASE  
**Prerequisites**: Phase 0, 1 & 2 complete

### Overview
Implement protocol detection for CTAP2, U2F, PIV, OpenPGP, OTP, and NDEF. Update Protocols page to display real detection results instead of placeholder data.

### Current State Analysis
Before starting, verify:
1. Device connection works (can open/close devices)
2. HID and APDU transport work correctly  
3. protocol.rs exists with placeholder detection

### Tasks for Phase 3

#### TASK 3.1: Implement Protocol Detection Logic
Update `/native/src/protocol.rs` to detect each protocol:
- FIDO2/CTAP2: Send CTAP2 getInfo command
- U2F/CTAP1: Send U2F version command
- PIV: Try PIV SELECT APDU (A0 00 00 03 08)
- OpenPGP: Try OpenPGP SELECT APDU
- OTP: Try vendor-specific OTP command
- NDEF: Try NDEF read command

#### TASK 3.2: Add detectProtocols RPC Command
Add handler in main.rs that:
- Takes deviceId parameter
- Calls detect_protocols from protocol.rs
- Returns ProtocolSupport structure

#### TASK 3.3: Update Protocols Page UI
Update `/web/src/pages/Protocols.tsx` to:
- Call detectProtocols RPC when device is selected
- Display real protocol support data
- Show protocol details and capabilities

---

## 🚀 REMAINING PHASES (4-13)

Due to file length, detailed instructions for remaining phases follow the same comprehensive pattern:

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
- [x] Phase 2: Device Connection & Transport Layer

### In Progress
- [ ] Phase 3: Protocol Detection (CURRENT)

### Remaining
- [ ] Phases 4-13

---

**END OF AGENTS.MD**

**Version**: 3.1  
**Last Updated**: November 18, 2025  
**Status**: Phase 2 Complete, Phase 3 Ready
