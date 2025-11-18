# AGENTS.md — Feitian SK Manager WebApp
## Chrome Extension + Native Messaging Host Architecture

---

## 1. Project Overview

This repository builds a browser-first **Feitian Security Key (SK) Manager** with full support for:

- **FIDO2** (CTAP2)
- **U2F** (CTAP1)
- **PIV** (CCID smart card)
- **OpenPGP** (CCID)
- **OTP** (HOTP only - no TOTP for now)
- **NDEF** (NFC Data Exchange Format)
- **Vendor/firmware commands**

Modern browsers do not allow direct access to security keys for these operations because CTAP HID + CCID interfaces are blocked by WebUSB/WebHID.

Therefore, we use the same architecture as Yubico, SoloKey, and Ledger:

**Web UI → Chrome Extension → Native Messaging Host → Device**

This architecture provides:
- Full privileged access to HID + CCID
- A fully web-based UI
- Minimal user installation friction
- Secure, auditable communication chain

---

## 2. High-Level Architecture

```
Web UI (React, Vite, TypeScript)
        ⇅ window.postMessage / content script
Chrome Extension (MV3, background service worker)
        ⇅ chrome.runtime.connectNative()
Native Host (Rust binary, JSON-RPC protocol)
        ⇅ PC/SC (CCID), HIDAPI
Feitian Security Key (Vendor ID: 0x096e)
```

### Roles:

**Web UI**
- Entire user interface
- Sends requests to extension
- Renders device data, flows, operations
- Auto-detects missing components and guides installation

**Chrome Extension**
- Bridge between website and native host
- Safely validates & forwards requests
- No origin restrictions (extension-based architecture)
- No protocol logic, only message routing

**Native Host**
- Actual device access
- CTAPHID, CCID APDUs, OTP vendor commands
- JSON-RPC server over stdin/stdout
- Input validation + security enforcement
- **Filters devices by Vendor ID 0x096e (Feitian only)**

---

## 3. Technology Stack

### Frontend
- React
- Vite
- TypeScript
- Minimal black/white theme

### Chrome Extension
- Manifest v3
- Background service worker
- Message passing bridge
- **Distribution**: Chrome Web Store + Edge Add-ons

### Native Host
- **Language**: Rust
- **Libraries**:
  - `pcsc` (for CCID/PIV/OpenPGP)
  - `hidapi` (for FIDO2/U2F/OTP)
  - `serde_json` (for JSON-RPC)
  - `tokio` (async runtime)

### Packaging
- Windows: `.exe` installer (no code signing)
- macOS: `.pkg` or `.dmg` (no code signing)
- Linux: `.deb` + `.rpm`

---

## 4. UI/UX Design Specifications

### Layout Architecture
- **Left Sidebar Navigation**: Static vertical navigation menu on the left side
- **Main Content Area**: Right side of screen for dynamic content display
- **Two-column layout**: Navigation fixed, content scrollable

### Design System

#### Color Scheme
- **Primary Background**: Pure white (`#FFFFFF`) or pure black (`#000000`)
- **Secondary Background**: Light gray (`#F5F5F5`) or dark gray (`#1A1A1A`)
- **Accent Colors**: Minimal use of gray shades for hierarchy
- **Text**: High contrast black on white or white on black
- **Borders**: Subtle gray strokes for separation

#### Typography
- **Font Family**: Modern sans-serif (Inter, SF Pro, or Segoe UI)
- **Heading Sizes**: 
  - H1: 32px (Page titles)
  - H2: 24px (Section headers)
  - H3: 18px (Subsection headers)
- **Body Text**: 14-16px
- **Labels**: 12-14px
- **Font Weight**: Regular (400) and Semibold (600)

#### Border Radius
- **Large Elements**: 24px corner radius (cards, panels, major containers)
- **Medium Elements**: 16px corner radius (buttons, input fields)
- **Small Elements**: 12px corner radius (badges, chips)
- **Consistent smooth curves** throughout the entire interface

#### Spacing System
- **Base Unit**: 8px
- **Component Padding**: 16px-24px
- **Section Margins**: 32px-48px
- **Element Gaps**: 12px-16px

### Animation & Transitions

#### Smooth Transitions
- **Tab Switching**: 300ms ease-in-out with fade + slide
- **Button Hover**: 150ms ease-out scale (1.02x) + shadow
- **Modal/Dialog**: 250ms ease-in-out with backdrop fade
- **Accordion Expand**: 300ms ease-in-out height transition
- **Loading States**: Skeleton screens with shimmer effect

#### Interactive States
- **Hover**: Subtle scale, shadow, or background change
- **Active/Pressed**: Slight scale down (0.98x)
- **Focus**: Distinct outline with accent color
- **Disabled**: 40% opacity with cursor-not-allowed

### Navigation Sidebar

#### Structure
- **Width**: 240px fixed
- **Background**: White or black (theme dependent)
- **Position**: Fixed left, full height
- **Border**: 1px subtle right border

#### Navigation Items
- **Height**: 48px per item
- **Padding**: 16px horizontal
- **Border Radius**: 12px (when active/hover)
- **Active State**: Bold text + background tint
- **Hover State**: Light background tint + smooth transition
- **Icon + Text**: Left-aligned icon (20px) + label with 12px gap

#### Navigation Menu
1. Dashboard (Home icon)
2. Device Manager (USB icon)
3. FIDO2 (Shield icon)
4. U2F (Key icon)
5. PIV (Card icon)
6. OpenPGP (Lock icon)
7. OTP (Clock icon)
8. NDEF (Tag icon)
9. Debug Console (Terminal icon)
10. Settings (Gear icon)

### Content Area Design

#### Page Header
- **Height**: 80px
- **Elements**: Page title (H1) + breadcrumb + device status badge
- **Bottom Border**: 1px separator line

#### Content Sections
- **Card-Based Layout**: Each functional group in a rounded card
- **Card Shadow**: Subtle elevation (0px 2px 8px rgba(0,0,0,0.08))
- **Card Padding**: 24px
- **Card Margin**: 24px between cards
- **Card Border Radius**: 24px

#### Buttons
- **Primary Button**: Solid black (or white in dark mode), 16px radius, 12px padding vertical, 24px horizontal
- **Secondary Button**: Outlined with border, transparent background
- **Danger Button**: Red accent for destructive actions
- **Icon Buttons**: 40x40px, centered icon, 12px radius
- **Button Groups**: 8px gap between related buttons

#### Form Elements
- **Input Fields**: 
  - Height: 44px
  - Border: 1px solid gray
  - Border Radius: 12px
  - Padding: 12px 16px
  - Focus: Thicker border + subtle shadow
- **Dropdowns**: Same style as input fields
- **Checkboxes/Toggles**: Modern toggle switches with smooth slide animation
- **Labels**: Above input fields, 12px margin bottom

### Big Tech Style Elements

#### Apple-Style Minimalism
- Generous white space
- Clean typography hierarchy
- Subtle shadows and depth
- Smooth, natural animations

#### Google Material Influence
- Card-based layouts
- Elevation system via shadows
- Ripple effects on click (optional)
- Clear visual feedback

#### Microsoft Fluent Design
- Acrylic blur effects (optional for overlays)
- Subtle depth cues
- Consistent rounded corners
- Thoughtful motion

### Responsive Behavior
- **Sidebar**: Collapsible to icon-only mode on smaller screens
- **Content**: Fluid width, adapts to available space
- **Mobile**: Stack vertically, hamburger menu for navigation
- **Breakpoints**: 1440px, 1024px, 768px, 375px

---

## 5. Functional Scope (Based on Reference Images)

### Device Protocol Detection
The application automatically detects which protocols the connected Feitian device supports:
- FIDO2 (CTAP2)
- U2F (CTAP1)
- PIV
- OpenPGP
- OTP (HOTP)
- NDEF

Users can then select which protocol(s) to enable/disable on the device.

### FIDO2 / CTAP2

**Reference**: Image 1 shows the FIDO2 management interface

#### PIN Management
- **Change PIN**:
  - Input: Current PIN (masked input)
  - Input: New PIN (masked input)
  - Input: Confirm New PIN (masked input)
  - Validation: PINs match, meets requirements (6-63 characters)
  - Submit button with confirmation dialog
  - Success/error feedback

#### Credential Management
- **Enumerate Credentials** (if device supports):
  - Display list of stored credentials
  - Show: RP ID (Relying Party), User ID, Credential ID
  - Sortable and filterable list
  - Credential count display
- **Delete Credential**:
  - Select credential from list
  - Confirm deletion dialog (with warning)
  - Requires PIN authentication
  - Success/error feedback

#### Reset Operations
- **Reset Device**:
  - Big, prominent "Reset Device" button
  - Multi-step confirmation (requires typing "RESET" or similar)
  - Warning about data loss (all credentials, PIN, etc.)
  - Requires physical touch confirmation on device
  - Progress indicator during reset
  - Success confirmation with next steps

#### Additional Features
- GetInfo display (AAGUID, firmware version, supported features)
- PIN retry counter display (informational only, no lockout warnings)
- User presence indicator (shows when touch is required)
- Timeout handling for operations requiring touch

### U2F / CTAP1
- Register
- Authenticate
- Version query

### PIV (Smart Card / CCID)

**Reference**: Image 2 shows the PIV management interface

#### PIN Management
- **Change PIN**:
  - Input: Current PIN
  - Input: New PIN
  - Input: Confirm New PIN
  - No lockout warnings (unlimited retries)
  - Display retry counter (informational only)
  
- **Change PUK** (PIN Unblocking Key):
  - Input: Current PUK
  - Input: New PUK
  - Input: Confirm New PUK
  - Display retry counter
  
- **Change Manager Key**:
  - Input: Current Manager Key
  - Input: New Manager Key
  - Input: Confirm New Manager Key
  - Typically 24-byte key (hex input)

#### Certificate Management

**Slot Selector**:
- Dropdown menu with all PIV slots:
  - Authentication (9a)
  - Digital Signature (9c)
  - Key Management (9d)
  - Card Authentication (9e)
  - Attestation (f9)
  - Additional retired key slots (82-95)

**Policy Manager**:
- Set certificate usage policies
- Configure PIN requirements for slot
- Set touch requirements

**Certificate Operations**:

1. **Export Certificate**:
   - Select slot from dropdown
   - Download certificate as .cer, .pem, or .der
   - Show certificate info before export
   
2. **Delete Certificate**:
   - Select slot from dropdown
   - Confirm deletion dialog
   - Requires PIN authentication
   - Clear slot data

3. **Import Certificate**:
   - Select slot from dropdown
   - Upload certificate file (.cer, .pem, .der, .crt)
   - Validate certificate format
   - Requires Manager Key or PIN
   - Show import preview

4. **Generate Key Pair**:
   - Select slot from dropdown
   - Choose algorithm (RSA 2048, RSA 3072, RSA 4096, ECC P-256, ECC P-384)
   - Choose PIN policy (Always, Once, Never)
   - Choose touch policy (Always, Cached, Never)
   - Generate key on device
   - Export public key option
   - Progress indicator

**Certificate Info Display**:
- Subject DN
- Issuer DN
- Serial Number
- Valid From / Valid To
- Key Algorithm and Size
- Fingerprint (SHA-1, SHA-256)
- Display: "No certificate loaded" for empty slots

#### Reset Operations
- **Reset Device**: Factory reset (warning dialog with confirmation)

#### MAC Smart Card Logon
- **Set Up**: Configure PIV credentials for macOS login
- Import certificate and configure system preferences
- Test authentication flow

### OpenPGP (CCID)
- Read card data
- Import/export keys
- Set user data
- Change PIN/Admin PIN

### OTP (HOTP Only)

**Reference**: Image 3 shows the OTP slot configuration interface

#### Slot Management

**Two Slots Available**:
- **Slot 1 (Short Touch)**: Triggered by short press of button
- **Slot 2 (Long Touch)**: Triggered by long press (2+ seconds) of button

**Slot Status Display**:
- Show if slot is "configured" or "empty"
- Display current configuration summary (if configured)

#### Slot Configuration

**Configure Button** (per slot):

1. **HOTP Settings**:
   - **Secret Key Input**: 
     - Format selector dropdown (Base32, Hex, Base64, Plain Text, CSV)
     - Text input for manual entry
     - OR "Generate Seed" button for secure random generation
   - **Counter**: Starting counter value (default 0)
   - **Digits**: 6 or 8 digit OTP output
   - **Name/Label**: Friendly name for the configuration

2. **Advanced Options**:
   - Token ID (optional)
   - OATH compliance mode
   - Button behavior settings

3. **Actions**:
   - Save configuration button
   - Test OTP generation
   - Cancel button

**Delete Button** (per slot):
- Clear slot configuration
- Confirmation dialog: "Are you sure? This will erase the OTP configuration."
- Immediate effect after confirmation

**Swap Function**:
- Swap button between Slot 1 and Slot 2 sections
- Exchanges the complete configuration between slots
- Visual animation showing the swap
- Confirmation dialog before swap

#### Seed Format Support

1. **Base32** (RFC 4648):
   - Standard TOTP/HOTP format
   - Case insensitive
   - Validation for valid base32 characters

2. **Hex** (Hexadecimal):
   - 0-9, A-F characters
   - Even number of characters
   - Commonly used in technical contexts

3. **Base64** (RFC 4648):
   - Standard base64 encoding
   - Validation for padding and valid characters

4. **Plain Text**:
   - UTF-8 string converted to bytes
   - Warning about shorter effective key length

5. **CSV** (Comma-Separated Values):
   - Parse seed from CSV format
   - Extract relevant fields
   - Common in bulk imports

#### Seed Generator
- **Generate Secure Seed** button
- Uses cryptographically secure random number generator
- Default output: Base32 format (for compatibility)
- Automatically fills secret key input
- Copy-to-clipboard functionality
- Option to regenerate

#### OTP Testing
- **Test Generation** button (when slot configured)
- Simulates button press
- Displays generated OTP code
- Shows counter increment (for HOTP)
- Copy OTP to clipboard
- Validates against expected output (if reference provided)

#### Visual Feedback
- Slot configuration cards with distinct styling
- Configured slots: Highlighted with accent color
- Empty slots: Muted/disabled appearance
- Touch type indicator: Visual badge showing "Short Touch" or "Long Touch"

### NDEF
- Read NDEF data
- Write NDEF data
- Configure NDEF behavior

### Vendor Operations
- Firmware version
- Vendor APDUs
- Factory reset (if supported)
- Protocol enable/disable

---

## 6. UI Implementation Notes

### Critical UI Requirements

1. **Do NOT Copy Reference UI Design**: 
   - The reference images (FIDO2, PIV, OTP) show the **functionality only**
   - We are implementing our own modern, black-and-white big tech style UI
   - Use reference images for feature requirements, not visual design

2. **Layout Structure**:
   - Static sidebar navigation on the left (240px width)
   - Main content area on the right (fluid width)
   - No top navigation bar
   - Full-height layout

3. **Design Philosophy**:
   - Apple/Google/Microsoft inspired minimalism
   - High contrast black and white
   - Generous white space
   - Large corner radius (24px for cards, 16px for buttons)
   - Smooth animations and transitions

4. **Component Library**:
   - Build reusable components for consistency
   - Shared button styles, input fields, cards
   - Centralized animation/transition definitions
   - Design tokens for colors, spacing, typography

5. **Polish Phase** (Post-MVP):
   - Advanced animations and micro-interactions
   - Smooth tab transitions with fade + slide
   - Button hover effects with scale and shadow
   - Loading skeleton screens
   - Success/error toast notifications with animations
   - Modal dialogs with backdrop blur

### Component Hierarchy

```
App Layout
├── Sidebar Navigation (Static)
│   ├── Logo/Brand
│   ├── Device Status Badge
│   ├── Navigation Menu
│   │   ├── Dashboard
│   │   ├── Device Manager
│   │   ├── FIDO2
│   │   ├── U2F
│   │   ├── PIV
│   │   ├── OpenPGP
│   │   ├── OTP
│   │   ├── NDEF
│   │   ├── Debug Console
│   │   └── Settings
│   └── Footer (Version, Help)
│
└── Main Content Area
    ├── Page Header
    │   ├── Page Title
    │   ├── Breadcrumb
    │   └── Quick Actions
    │
    └── Content Sections (Cards)
        ├── Section Header
        ├── Section Content
        └── Section Actions
```

### Animation Specifications

#### Page Transitions
```typescript
const pageTransition = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
  transition: { duration: 0.3, ease: 'easeInOut' }
}
```

#### Button Interactions
```typescript
const buttonHover = {
  scale: 1.02,
  boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
  transition: { duration: 0.15 }
}

const buttonTap = {
  scale: 0.98
}
```

#### Card Animations
```typescript
const cardEntry = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.4, ease: 'easeOut' }
}
```

### Accessibility Requirements

- **Keyboard Navigation**: All interactive elements accessible via keyboard
- **Focus Indicators**: Clear, high-contrast focus outlines
- **ARIA Labels**: Proper ARIA attributes for screen readers
- **Color Contrast**: WCAG AA compliance (4.5:1 minimum)
- **Loading States**: Clear feedback for async operations
- **Error Messages**: Descriptive, actionable error text

### Development Guidelines for AI Agents

When implementing UI components, AI agents should:

1. **Start with functionality first**, styling second
2. **Use semantic HTML** (header, nav, main, section, article)
3. **Implement responsive behavior** from the start
4. **Add smooth transitions** to all interactive elements
5. **Use CSS custom properties** for theming and consistency
6. **Test keyboard navigation** for all workflows
7. **Include loading and error states** for all async operations
8. **Add micro-interactions** for better user feedback

---

## 7. AI Agent Implementation Phases

Each phase includes specific AI agent prompts and manual steps required.

### Phase 0 — Repository Foundation & Setup

**AI Agent Tasks**:
1. Create monorepo structure with `/web`, `/extension`, `/native`, `/docs`
2. Initialize React + Vite + TypeScript project in `/web`
3. Create basic Chrome Extension manifest v3 in `/extension`
4. Create Rust project skeleton in `/native` with cargo workspace
5. Add `.gitignore`, `README.md`, `LICENSE`
6. Set up GitHub Actions CI for building all components

**Manual Steps**:
- [ ] Create GitHub repository
- [ ] Review and commit generated structure
- [ ] Verify CI builds successfully

**Deliverable**: Working build system for all three components

---

### Phase 1 — Native Host Foundation

**AI Agent Tasks**:
1. Implement JSON-RPC I/O loop (stdin/stdout) in Rust native host
2. Implement `ping` command (echo test)
3. Implement `getVersion` command (returns host version)
4. Implement `listDevices` command:
   - Enumerate HID devices (hidapi)
   - Enumerate CCID readers (pcsc)
   - Filter by Vendor ID `0x096e`
   - Return device info (VID, PID, type, path)
5. Add error handling and logging
6. Create native host manifest file for Chrome

**Manual Steps**:
- [ ] Test native host binary standalone (`echo '{"command":"ping"}' | ./native-host`)
- [ ] Place native host manifest in correct location for Chrome/Edge

**Deliverable**: Native host binary that can enumerate Feitian devices

---

### Phase 2 — Extension Bridge

**AI Agent Tasks**:
1. Implement background service worker with native messaging connection
2. Add content script that injects `window.chromeBridge` API
3. Implement message queue and request/response matching
4. Add RPC forwarding functions:
   - `ping()`
   - `getVersion()`
   - `listDevices()`
5. Add connection status monitoring
6. Handle native host crashes gracefully

**Manual Steps**:
- [ ] Load unpacked extension in Chrome
- [ ] Test extension → native host connection
- [ ] Verify device enumeration works

**Deliverable**: Working extension that can communicate with native host

---

### Phase 3 — Web UI Foundation with Modern Design System

**AI Agent Tasks**:
1. Create React app layout with modern black/white theme:
   - **Two-column layout**: Static left sidebar (240px) + fluid right content area
   - Implement navigation sidebar with menu items
   - Create page header component
   - Set up routing for all major pages
   
2. **Design System Setup**:
   - Create CSS custom properties for:
     - Colors (black, white, gray scales)
     - Spacing system (8px base unit)
     - Typography scale
     - Border radius tokens (24px, 16px, 12px)
     - Shadow tokens
   - Set up animation/transition utilities
   - Create base component styles

3. **Core UI Components**:
   - Button component (primary, secondary, danger variants)
   - Input field component (with focus states)
   - Card component (24px border radius, subtle shadow)
   - Badge component (for device status)
   - Toast notification component
   - Modal/Dialog component
   - Loading spinner/skeleton screens

4. **Sidebar Navigation**:
   - Logo/brand area at top
   - Device connection status badge
   - Navigation menu items with icons:
     - Dashboard, Device Manager, FIDO2, U2F, PIV, OpenPGP, OTP, NDEF, Debug Console, Settings
   - Active state highlighting (bold + background tint)
   - Smooth hover transitions
   - Footer with version and help links

5. **Main Content Area**:
   - Page header with title and breadcrumb
   - Content cards with consistent styling
   - Responsive layout that adapts to content

6. Implement "Extension Status" component:
   - Detects if extension is installed
   - Detects if native host is reachable
   - Shows installation guide if missing
   - Modern card-based status display

7. Create "Device Manager" page:
   - Shows list of connected Feitian devices
   - Single device selection (one at a time)
   - Connect/disconnect buttons with smooth transitions
   - Device info display panel

8. Add basic error handling and user feedback:
   - Toast notifications for success/error
   - Loading states for all operations
   - Empty states with helpful messages

**UI/UX Requirements**:
- Use 24px border radius for cards
- Use 16px border radius for buttons
- Implement smooth transitions (300ms ease-in-out for tab switching)
- High contrast black and white color scheme
- Generous spacing and white space
- Clean typography hierarchy
- NO emulation of reference UI design

**Manual Steps**:
- [ ] Run web UI locally (`npm run dev`)
- [ ] Test device detection with real hardware
- [ ] Verify navigation works smoothly
- [ ] Test responsive behavior at different screen sizes
- [ ] Verify all animations are smooth

**Deliverable**: Web UI with modern design system and device detection working

---

### Phase 4 — HID & CCID Transport Layer

**AI Agent Tasks**:
1. Implement in native host:
   - `openDevice(deviceId)` - opens HID or CCID connection
   - `closeDevice(deviceId)` - closes connection
   - `sendHid(deviceId, data)` - sends raw HID packet
   - `receiveHid(deviceId, timeout)` - receives HID packet
   - `transmitApdu(deviceId, apdu)` - sends CCID APDU
   - CTAPHID framing (packet assembly/disassembly)
2. Add these RPC commands to extension bridge
3. Create "Debug Console" page in web UI:
   - Raw HID packet sender
   - Raw APDU sender
   - Hex log viewer with timestamps

**Manual Steps**:
- [ ] Test raw HID communication with real device
- [ ] Test APDU communication with PIV applet

**Deliverable**: End-to-end raw communication working

---

### Phase 5 — Protocol Detection

**AI Agent Tasks**:
1. Implement `detectProtocols(deviceId)` in native host:
   - Try CTAP2 `getInfo` → FIDO2 supported
   - Try CTAP1 version → U2F supported
   - Try PIV SELECT → PIV supported
   - Try OpenPGP SELECT → OpenPGP supported
   - Try OTP vendor command → OTP supported
   - Try NDEF read → NDEF supported
2. Return bitmask or list of supported protocols
3. Create "Protocol Manager" UI:
   - Shows detected protocols
   - Enable/disable toggles for each protocol
4. Implement protocol enable/disable vendor commands

**Manual Steps**:
- [ ] Test with different Feitian device models
- [ ] Verify protocol detection accuracy

**Deliverable**: Automatic protocol detection and enable/disable

---

### Phase 6 — FIDO2 Implementation

**AI Agent Tasks**:
1. Implement CTAP2 commands in native host:
   - `ctap2_getInfo(deviceId)`
   - `ctap2_clientPinSet(deviceId, newPin)`
   - `ctap2_clientPinChange(deviceId, currentPin, newPin)`
   - `ctap2_clientPinGetRetries(deviceId)`
   - `ctap2_reset(deviceId)` - requires user presence
   - `ctap2_enumerateCredentials(deviceId, pin)` (if supported)
   - `ctap2_deleteCredential(deviceId, credentialId, pin)`

2. Create "FIDO2 Manager" page in web UI with **modern design**:

   **Page Structure** (card-based sections):
   
   a. **Authenticator Info Card**:
      - Display AAGUID
      - Firmware version
      - Supported features/versions
      - Max credential count
      - PIN retry counter (display only)
   
   b. **PIN Management Card**:
      - **Change PIN** form:
        - Current PIN input (masked)
        - New PIN input (masked)
        - Confirm New PIN input (masked)
        - Validation: 6-63 characters, matching
        - Submit button with loading state
        - Success/error toast notification
      - PIN requirements displayed clearly
      - Retry counter display (informational, no warnings)
   
   c. **Credential Management Card** (if supported):
      - Credential count display
      - Enumerate button (requires PIN)
      - Table/List view:
        - RP ID (Relying Party domain)
        - User ID / Username
        - Credential ID (truncated with copy button)
        - Created date (if available)
      - Search/filter functionality
      - Sort by RP ID or date
      - **Delete button** per credential:
        - Inline delete icon
        - Confirmation modal: "Delete credential for [RP ID]?"
        - Requires PIN re-entry
        - Success/error feedback
   
   d. **Reset Device Card**:
      - Large, prominent "Reset Device" button (danger style)
      - Warning text about complete data loss
      - Multi-step confirmation flow:
        1. Click Reset → Show warning modal
        2. Type "RESET" in confirmation input
        3. Click "I understand, reset device"
        4. Wait for physical touch on device
        5. Show progress indicator
        6. Display success message with next steps
      - Disable during operation

3. Handle user presence prompts:
   - Show "Touch your security key" notification
   - Animate touch indicator
   - Timeout handling (30 seconds typical)
   - Clear feedback when touch detected

4. **UI/UX Details**:
   - Each section in a separate card (24px border radius)
   - Smooth animations between states
   - Loading spinners for async operations
   - Toast notifications for success/error
   - Proper form validation with inline errors
   - Disabled states for buttons during operations
   - Modal dialogs for confirmations (with backdrop)

**Reference Functionality** (from Image 1):
- PIN Management: Change PIN
- Credential Management: Enum Credential
- Reset: Reset Device

**Manual Steps**:
- [ ] Test PIN set operation (on device with no PIN)
- [ ] Test PIN change operation
- [ ] Test credential enumeration (if device supports)
- [ ] Test credential deletion
- [ ] Test reset operation (WARNING: destructive)
- [ ] Verify touch prompts appear correctly
- [ ] Test all error cases (wrong PIN, timeout, etc.)

**Deliverable**: Full FIDO2 management functionality with modern UI

---

### Phase 7 — U2F Implementation

**AI Agent Tasks**:
1. Implement CTAP1 commands in native host:
   - `u2f_register(deviceId, challenge, application)`
   - `u2f_authenticate(deviceId, challenge, application, keyHandle)`
   - `u2f_version(deviceId)`
2. Create "U2F Manager" page in web UI:
   - Version display
   - Test registration/authentication
   - Key handle management

**Manual Steps**:
- [ ] Test U2F registration flow
- [ ] Test authentication with registered key

**Deliverable**: U2F protocol support

---

### Phase 8 — PIV Implementation

**AI Agent Tasks**:
1. Implement PIV commands in native host:
   - `piv_select(deviceId)`
   - `piv_verifyPin(deviceId, pin)`
   - `piv_changePin(deviceId, currentPin, newPin)`
   - `piv_changePuk(deviceId, currentPuk, newPuk)`
   - `piv_changeManagerKey(deviceId, currentKey, newKey)`
   - `piv_getRetries(deviceId)` - returns PIN/PUK retry counters
   - `piv_generateKey(deviceId, slot, algorithm, pinPolicy, touchPolicy, pin)`
   - `piv_importCert(deviceId, slot, certData, managementKey, pin)`
   - `piv_readCert(deviceId, slot)`
   - `piv_exportCert(deviceId, slot, format)`
   - `piv_deleteCert(deviceId, slot, managementKey, pin)`

2. Create "PIV Manager" page in web UI with **modern design**:

   **Page Structure** (card-based sections):
   
   a. **PIN Management Card**:
      - Three horizontal buttons in a row:
        - **Change PIN**
        - **Change PUK**
        - **Change Manager Key**
      - Display retry counters below buttons (informational)
      - No lockout warnings
      
      **Change PIN Modal**:
      - Current PIN input
      - New PIN input
      - Confirm New PIN input
      - PIN requirements (6-8 digits)
      - Submit with validation
      
      **Change PUK Modal**:
      - Current PUK input
      - New PUK input
      - Confirm New PUK input
      - PUK requirements (8 digits)
      
      **Change Manager Key Modal**:
      - Current Manager Key input (hex)
      - New Manager Key input (hex, 24 bytes)
      - Confirm New Manager Key input
      - Validation for hex format and length
   
   b. **Certificate Management Card**:
      
      **Slot Selector Section**:
      - Prominent dropdown: "Slot: [Authentication (9a)]"
      - Options:
        - Authentication (9a)
        - Digital Signature (9c)
        - Key Management (9d)
        - Card Authentication (9e)
        - Attestation (f9)
        - Retired Key Slot 1-20 (82-95)
      
      **Policy Manager Section**:
      - Button to open policy configuration
      - Settings:
        - PIN policy (Always/Once/Never)
        - Touch policy (Always/Cached/Never)
      
      **Action Buttons Grid** (2x2 layout):
      - **Export** (with download icon)
        - Opens modal to select format (.cer, .pem, .der)
        - Downloads certificate file
      - **Delete** (with trash icon)
        - Confirmation modal with warning
        - Requires PIN or Manager Key
      - **Import** (with upload icon)
        - File picker for certificate
        - Validation of format
        - Requires Manager Key
        - Shows preview before import
      - **Generate** (with refresh icon)
        - Opens key generation wizard modal
        
      **Generate Key Wizard**:
      - Step 1: Select algorithm
        - RSA 2048, RSA 3072, RSA 4096
        - ECC P-256, ECC P-384
      - Step 2: Select policies
        - PIN policy dropdown
        - Touch policy dropdown
      - Step 3: Confirmation
        - Summary of selections
        - Generate button
        - Progress indicator (key generation can take 5-30 seconds)
      - Step 4: Success
        - Option to export public key
        - Option to import certificate now
   
   c. **Certificate Info Panel** (right side or below):
      - Large display area showing:
        - "No certificate loaded." (when empty)
        - OR Certificate details:
          - Subject DN
          - Issuer DN
          - Serial Number
          - Valid From / Valid To dates
          - Key Algorithm and Size
          - SHA-1 Fingerprint
          - SHA-256 Fingerprint
      - Formatted as readable key-value pairs
      - Syntax highlighting for DNs
   
   d. **Reset Card**:
      - "Reset Device" button (danger style)
      - Same confirmation flow as FIDO2
   
   e. **MAC Smart Card Logon Card**:
      - "Set up" button/section
      - Instructions for macOS configuration
      - Link to system preferences
      - Test authentication button

3. **Certificate Parsing**:
   - Parse X.509 certificates in native host or web UI
   - Display human-readable certificate information
   - Validate certificate format on import

4. **UI/UX Details**:
   - Dropdown for slot selection with visual highlighting
   - Button grid with icons and labels
   - Modal wizards for complex operations
   - Certificate info panel with monospace font for fingerprints
   - Smooth transitions when switching slots
   - Loading states during key generation (long operation)
   - File upload with drag-and-drop support
   - Success animations for completed operations

**Reference Functionality** (from Image 2):
- PIN Management: Change PIN, Change PUK, Change Manager Key
- Cert Management: Slot selector, Export, Delete, Import, Generate
- Policy Manager: Configure slot policies
- Certificate Info: Display certificate details
- MAC Smart Card Logon: Setup functionality

**Manual Steps**:
- [ ] Test PIN verification and change
- [ ] Test PUK change
- [ ] Test Manager Key change
- [ ] Test key generation in different slots with different algorithms
- [ ] Test certificate import (create test certificate first)
- [ ] Test certificate export in all formats
- [ ] Test certificate deletion
- [ ] Verify certificate info display is accurate
- [ ] Test policy configuration
- [ ] Test on macOS for Smart Card logon (if applicable)

**Deliverable**: Full PIV smart card functionality with modern UI

---

### Phase 9 — OpenPGP Implementation

**AI Agent Tasks**:
1. Implement OpenPGP commands in native host:
   - `openpgp_select(deviceId)`
   - `openpgp_readData(deviceId)` - card holder, URL, etc.
   - `openpgp_verifyPin(deviceId, pin)`
   - `openpgp_changePin(deviceId, currentPin, newPin)`
   - `openpgp_changeAdminPin(deviceId, currentPin, newPin)`
   - `openpgp_importKey(deviceId, keyData, adminPin)`
   - `openpgp_exportPublicKey(deviceId)`
2. Create "OpenPGP Manager" page in web UI:
   - Card info display
   - PIN management
   - Key import/export interface

**Manual Steps**:
- [ ] Test OpenPGP applet access
- [ ] Test key import/export

**Deliverable**: OpenPGP card functionality

---

### Phase 10 — OTP (HOTP) Implementation

**AI Agent Tasks**:
1. Implement OTP commands in native host:
   - `otp_readSlot(deviceId, slotNumber)` - returns slot configuration
   - `otp_writeSlot(deviceId, slotNumber, config)` - writes HOTP configuration
   - `otp_deleteSlot(deviceId, slotNumber)` - clears slot
   - `otp_swapSlots(deviceId)` - swaps Slot 1 and Slot 2
   - `otp_generateSeed(length)` - secure random seed generation
   - `otp_testGenerate(deviceId, slotNumber)` - test OTP generation
   - Seed format conversion functions: Base32 ↔ Hex ↔ Base64 ↔ Plain ↔ CSV

2. Create "OTP Manager" page in web UI with **modern design**:

   **Page Structure** (two-slot layout):
   
   a. **Slot 1 Card** (Short Touch):
      - Header: "SHORT TOUCH (Slot 1)"
      - Status indicator: "This slot is configured" OR "This slot is empty"
      - Configuration summary (when configured):
        - Name/Label
        - Algorithm: HOTP
        - Digits: 6 or 8
        - Counter value
      
      **Action Buttons**:
      - **Configure** button
        - Opens configuration modal
      - **Delete** button
        - Confirmation dialog
        - Clears slot immediately
   
   b. **Swap Button** (between slots):
      - Large centered button with up/down arrow icon: "⇅ Swap"
      - Swaps complete configuration between Slot 1 and Slot 2
      - Confirmation dialog: "Swap Slot 1 and Slot 2 configurations?"
      - Visual animation showing the swap
      - Only enabled when at least one slot is configured
   
   c. **Slot 2 Card** (Long Touch):
      - Header: "LONG TOUCH (Slot 2)"
      - Same structure as Slot 1
      - Status indicator
      - Configuration summary
      - Configure and Delete buttons

3. **Configure Slot Modal** (full-screen or large modal):
   
   **Secret Key Section**:
   - **Format Selector** dropdown:
     - Base32 (default)
     - Hex
     - Base64
     - Plain Text
     - CSV
   
   - **Secret Input**:
     - Large textarea or input field
     - Format validation based on selected type
     - Character counter
     - Real-time validation feedback
   
   - **Generate Seed Button**:
     - Prominent button: "Generate Secure Seed"
     - Uses cryptographically secure random generator
     - Automatically fills secret input in Base32 format
     - Option to regenerate
     - Copy-to-clipboard button next to generated seed
   
   **HOTP Configuration**:
   - **Name/Label** input:
     - Friendly name for this OTP
     - Max 64 characters
   
   - **Counter** input:
     - Starting counter value
     - Default: 0
     - Number input with validation
   
   - **Digits** selector:
     - Radio buttons or dropdown
     - Options: 6 digits or 8 digits
     - Default: 6
   
   - **Token ID** (optional):
     - Advanced option (collapsible section)
     - Hex input, 12 characters
   
   **Action Buttons**:
   - **Save Configuration**:
     - Validates all inputs
     - Writes to device
     - Shows success toast
     - Closes modal
   
   - **Test OTP Generation**:
     - Generates OTP with current config (without saving)
     - Displays generated code
     - Copy button
     - Shows counter increment
   
   - **Cancel**:
     - Closes modal without saving
     - Confirmation if changes made

4. **Seed Format Validation**:
   
   - **Base32**:
     - Valid characters: A-Z, 2-7
     - Case insensitive
     - Padding with '=' allowed
     - Show error for invalid characters
   
   - **Hex**:
     - Valid characters: 0-9, A-F, a-f
     - Must be even number of characters
     - Convert to bytes for device
   
   - **Base64**:
     - Standard Base64 alphabet
     - Padding validation
     - Show error for invalid characters
   
   - **Plain Text**:
     - UTF-8 string
     - Warning: "Plain text may result in shorter effective key length"
     - No validation needed
   
   - **CSV**:
     - Parse comma-separated values
     - Extract secret from appropriate field
     - Show preview of parsed data
     - Handle various CSV formats

5. **OTP Testing Interface**:
   - **Test Generate** button (when slot configured)
   - Modal showing:
     - Generated OTP code (large font)
     - Counter value used
     - Copy to clipboard button
     - Close button
   - Simulates physical button press
   - Does NOT increment device counter (read-only test)

6. **UI/UX Details**:
   - Two slot cards side-by-side on desktop, stacked on mobile
   - Distinct visual styling for Short Touch vs Long Touch
   - Configured slots: Highlighted background, accent border
   - Empty slots: Muted appearance, dashed border
   - Swap button: Animated icon rotation on click
   - Format selector: Clear labels with format descriptions
   - Seed input: Monospace font for better readability
   - Copy buttons: Toast notification on successful copy
   - Real-time validation with inline error messages
   - Loading states during write operations
   - Success animations after configuration save

**Reference Functionality** (from Image 3):
- Slot 1 (Short Touch): Configure, Delete
- Slot 2 (Long Touch): Configure, Delete
- Swap: Exchange Slot 1 and Slot 2
- Status: "configured" or "empty"

**Manual Steps**:
- [ ] Test HOTP configuration in both slots
- [ ] Test all seed formats: Base32, Hex, Base64, Plain, CSV
- [ ] Test seed generator functionality
- [ ] Test slot deletion with confirmation
- [ ] Test slot swap functionality
- [ ] Test OTP generation and verify codes
- [ ] Verify counter increments correctly
- [ ] Test format conversion and validation
- [ ] Test with actual authenticator apps (Google Authenticator, etc.)

**Deliverable**: Full HOTP configuration and management with modern UI

---

### Phase 11 — NDEF Implementation

**AI Agent Tasks**:
1. Implement NDEF commands in native host:
   - `ndef_read(deviceId)`
   - `ndef_write(deviceId, ndefData)`
   - `ndef_format(deviceId)`
2. Create "NDEF Manager" page in web UI:
   - NDEF data viewer
   - NDEF data editor (URL, text, etc.)
   - Format button

**Manual Steps**:
- [ ] Test NDEF read/write
- [ ] Test different NDEF record types

**Deliverable**: NDEF functionality

---

### Phase 12 — Packaging & Distribution

**AI Agent Tasks**:
1. Create build scripts for native host:
   - Windows: `.exe` with installer script
   - macOS: `.pkg`/`.dmg` with install script
   - Linux: `.deb` and `.rpm` packages
2. Update extension manifest with native host paths
3. Create extension build script for production
4. Create installation documentation
5. Add auto-updater check for native host (optional)

**Manual Steps for Chrome Web Store**:
- [ ] Create Chrome Web Store developer account ($5 fee)
- [ ] Prepare extension listing:
  - [ ] Write description
  - [ ] Create screenshots
  - [ ] Create promotional images
- [ ] Upload extension package
- [ ] Submit for review

**Manual Steps for Edge Add-ons**:
- [ ] Create Microsoft Partner Center account
- [ ] Prepare Edge listing (similar to Chrome)
- [ ] Upload extension package
- [ ] Submit for review

**Manual Steps for Distribution Website**:
- [ ] Create landing page with installation guide
- [ ] Host native host installers
- [ ] Create step-by-step installation video/GIF
- [ ] Add troubleshooting guide

**Deliverable**: Published extension + downloadable native host installers

---

### Phase 13 — Security Hardening

**AI Agent Tasks**:
1. Implement input validation:
   - APDU length limits
   - HID packet size limits
   - Command whitelist
2. Add rate limiting for sensitive commands:
   - PIN verification attempts tracking (display only)
   - Factory reset confirmation
3. Implement secure memory handling:
   - Zero sensitive data after use
   - No PIN/PUK logging
4. Add security audit logging
5. Implement extension ID verification in native host
6. Run security scanners:
   - `cargo clippy`
   - `cargo audit`
   - Static analysis tools

**Manual Steps**:
- [ ] Security review of all RPC commands
- [ ] Penetration testing with fuzzing tools
- [ ] Review all error messages for info leakage

**Deliverable**: Security-hardened production release

---

### Phase 14 — Testing & Documentation

**AI Agent Tasks**:
1. Create comprehensive test suite:
   - Unit tests for native host commands
   - Integration tests for extension bridge
   - E2E tests for web UI flows
2. Write user documentation:
   - Installation guide
   - User manual for each protocol
   - Troubleshooting guide
3. Write developer documentation:
   - Architecture overview
   - Build instructions
   - Contributing guidelines
   - JSON-RPC protocol specification
4. Create example workflows and video tutorials

**Manual Steps**:
- [ ] Test with all available Feitian device models
- [ ] User acceptance testing with real users
- [ ] Document any device-specific quirks

**Deliverable**: Fully tested and documented application

---

### Phase 15 — UI Polish & Advanced Animations

**AI Agent Tasks**:

This phase focuses on elevating the UI to big tech standards with advanced animations and micro-interactions.

1. **Advanced Page Transitions**:
   - Implement smooth fade + slide transitions between tabs
   - Add directional awareness (slide left when going back, right when going forward)
   - Staggered animations for card entry (each card animates in sequence)
   - Smooth scroll animations

2. **Button Micro-Interactions**:
   - Hover effects:
     - Scale transform (1.02x)
     - Box shadow growth
     - Subtle color shift
   - Active/pressed state:
     - Scale down (0.98x)
     - Reduced shadow
   - Success feedback:
     - Checkmark animation
     - Green pulse effect
   - Loading state:
     - Spinner integration
     - Disabled appearance

3. **Form Enhancements**:
   - Input field focus animations:
     - Border color transition
     - Label float animation
     - Subtle shadow glow
   - Validation feedback:
     - Real-time validation with smooth error appear/disappear
     - Success checkmark animation
     - Error shake animation
   - Password strength indicator with animated progress bar

4. **Modal & Dialog Animations**:
   - Backdrop fade-in (200ms)
   - Modal scale + fade entrance (250ms ease-out)
   - Exit animations in reverse
   - Smooth sheet slide-up on mobile

5. **Toast Notifications**:
   - Slide-in from top or bottom
   - Auto-dismiss with progress bar
   - Stack multiple toasts with slide animations
   - Different styles: success (green), error (red), info (blue), warning (yellow)
   - Icon animations (checkmark, X, info symbol)

6. **Loading States**:
   - Skeleton screens for data loading:
     - Animated gradient shimmer effect
     - Match layout of actual content
   - Progress bars with smooth animations
   - Spinner with rotation easing
   - Pulsing dot indicators

7. **Sidebar Navigation Polish**:
   - Smooth active state transition
   - Ripple effect on click (optional)
   - Icon rotation/scale on hover
   - Badge animations for notifications
   - Collapse/expand animation with icon rotation

8. **Card Animations**:
   - Hover state: Slight elevation increase
   - Entry animations: Staggered fade + slide up
   - Flip animations for certificate cards (show info on flip)
   - Expand/collapse animations for accordion sections

9. **Certificate Display**:
   - Smooth reveal of certificate details
   - Syntax highlighting with fade-in
   - Copy button with success checkmark animation

10. **OTP Slot Swap Animation**:
    - Visual swap with cross-over animation
    - Temporary highlight of swapped slots
    - Smooth position exchange

11. **Corner Radius Consistency**:
    - Verify all elements use design tokens:
      - 24px for cards and large containers
      - 16px for buttons and medium elements
      - 12px for badges and small elements
    - Smooth transitions when radius changes on hover

12. **Responsive Animations**:
    - Adjust animation duration for mobile (faster)
    - Reduce motion for users with prefers-reduced-motion
    - Touch-friendly tap animations on mobile

13. **Performance Optimization**:
    - Use CSS transforms for animations (GPU accelerated)
    - Implement will-change for frequently animated elements
    - Lazy load animations for off-screen elements
    - Debounce resize events

14. **Accessibility**:
    - Ensure animations don't interfere with screen readers
    - Provide skip animation option
    - Respect prefers-reduced-motion media query
    - Maintain focus management during animations

**Animation Library Setup**:
- Consider using Framer Motion for React animations
- Create reusable animation variants
- Centralize animation timing and easing functions

**Testing**:
- [ ] Test all animations at 60fps
- [ ] Verify reduced motion mode works
- [ ] Test on low-end devices
- [ ] Ensure no animation jank
- [ ] Test touch interactions on mobile
- [ ] Verify all transitions feel smooth and natural

**Deliverable**: Polished, production-ready UI with smooth animations and big tech aesthetics

---

## 7. JSON-RPC Protocol Specification

### Request Format
```json
{
  "id": 1,
  "command": "commandName",
  "params": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

### Response Format
```json
{
  "id": 1,
  "status": "ok",
  "result": {
    "data": "..."
  }
}
```

### Error Response
```json
{
  "id": 1,
  "status": "error",
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

### Core Commands

#### System Commands
- `ping` - Health check
- `getVersion` - Returns native host version
- `listDevices` - Lists all Feitian devices (VID 0x096e)

#### Device Commands
- `openDevice(deviceId)` - Opens connection
- `closeDevice(deviceId)` - Closes connection
- `detectProtocols(deviceId)` - Detects available protocols

#### Transport Commands
- `sendHid(deviceId, data)` - Send raw HID packet
- `receiveHid(deviceId, timeout)` - Receive HID packet
- `transmitApdu(deviceId, apdu)` - Send CCID APDU

#### FIDO2 Commands
- `ctap2_getInfo(deviceId)`
- `ctap2_clientPinSet(deviceId, newPin)`
- `ctap2_clientPinChange(deviceId, currentPin, newPin)`
- `ctap2_clientPinGetRetries(deviceId)`
- `ctap2_reset(deviceId)`
- `ctap2_enumerateCredentials(deviceId, pin)`
- `ctap2_deleteCredential(deviceId, credentialId, pin)`

#### U2F Commands
- `u2f_register(deviceId, challenge, application)`
- `u2f_authenticate(deviceId, challenge, application, keyHandle)`
- `u2f_version(deviceId)`

#### PIV Commands
- `piv_select(deviceId)`
- `piv_verifyPin(deviceId, pin)`
- `piv_changePin(deviceId, currentPin, newPin)`
- `piv_changePuk(deviceId, currentPuk, newPuk)`
- `piv_getRetries(deviceId)`
- `piv_generateKey(deviceId, slot, algorithm, pin)`
- `piv_importCert(deviceId, slot, certData, pin)`
- `piv_readCert(deviceId, slot)`
- `piv_deleteCert(deviceId, slot, pin)`

#### OpenPGP Commands
- `openpgp_select(deviceId)`
- `openpgp_readData(deviceId)`
- `openpgp_verifyPin(deviceId, pin)`
- `openpgp_changePin(deviceId, currentPin, newPin)`
- `openpgp_changeAdminPin(deviceId, currentPin, newPin)`
- `openpgp_importKey(deviceId, keyData, adminPin)`
- `openpgp_exportPublicKey(deviceId)`

#### OTP Commands
- `otp_readSlot(deviceId, slotNumber)`
- `otp_writeSlot(deviceId, slotNumber, config)`
- `otp_generateSeed()`

#### NDEF Commands
- `ndef_read(deviceId)`
- `ndef_write(deviceId, ndefData)`
- `ndef_format(deviceId)`

---

## 8. Chrome Extension Architecture

### Manifest Permissions
```json
{
  "manifest_version": 3,
  "name": "Feitian SK Manager",
  "version": "1.0.0",
  "permissions": [
    "nativeMessaging",
    "storage"
  ],
  "background": {
    "service_worker": "background.js"
  },
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content.js"]
    }
  ]
}
```

### Background Worker
- Opens connection to native host on startup
- Maintains persistent connection
- Handles request/response queue with ID matching
- Validates all messages
- Implements reconnection logic on failure
- Relays results back to web UI via content script

### Content Script
- Injects `window.chromeBridge` object into webpage
- Provides API:
  - `chromeBridge.send(command, params)` - Returns Promise
  - `chromeBridge.onDisconnect` - Event handler
  - `chromeBridge.isConnected()` - Connection status
- Handles message serialization/deserialization

---

## 9. Web UI Structure

### Pages
1. **Dashboard** - Overview, device status, quick actions
2. **Device Manager** - Connect, detect protocols, device info
3. **FIDO2 Manager** - PIN, credentials, reset
4. **U2F Manager** - Registration, authentication testing
5. **PIV Manager** - Certificates, key generation, PIN/PUK
6. **OpenPGP Manager** - Card data, keys, PINs
7. **OTP Manager** - HOTP configuration, seed management
8. **NDEF Manager** - NDEF data read/write
9. **Debug Console** - Raw HID/APDU communication, logs
10. **Installation Guide** - Step-by-step setup instructions

### UI Components
- Device selector (single device at a time)
- Connection status indicator
- Protocol detection badges
- Operation logs panel
- Error/success toast notifications
- Loading states for all operations
- Modal confirmations for destructive actions

### Theme
- Minimal black and white design
- High contrast for readability
- Clear visual hierarchy
- Responsive layout

---

## 10. Device Filtering

### Vendor ID
- **Filter**: `0x096e` (Feitian Technologies)
- All non-Feitian devices are ignored
- Multiple Feitian devices can be detected, but only one can be active

### Supported Product IDs
The native host should recognize common Feitian PIDs:
- `0x0850` - ePass FIDO
- `0x0852` - ePass FIDO-NFC
- `0x0853` - BioPass FIDO
- `0x0854` - AllinPass FIDO
- `0x0856` - ePass K9 FIDO
- And other PIDs as discovered

---

## 11. Error Handling Strategy

### Native Host
- All errors return structured JSON error responses
- Error codes for different failure types:
  - `DEVICE_NOT_FOUND`
  - `DEVICE_BUSY`
  - `COMMUNICATION_ERROR`
  - `INVALID_PIN`
  - `OPERATION_DENIED`
  - `TIMEOUT`
  - `UNSUPPORTED_OPERATION`
- Detailed error messages for debugging
- No sensitive data in error messages

### Extension
- Catches all native host errors
- Forwards errors to web UI
- Handles native host crashes gracefully
- Automatic reconnection attempts

### Web UI
- User-friendly error messages
- Actionable error guidance
- Error logging for support
- Retry mechanisms where appropriate

---

## 12. Installation Flow

### First Time Setup
1. User visits web UI
2. UI detects extension not installed
3. Shows link to Chrome Web Store / Edge Add-ons
4. User installs extension
5. Extension detects native host not installed
6. Shows download links for native host installer
7. User downloads and runs installer
8. Installer places native host binary in system location
9. Installer registers native host manifest with Chrome/Edge
10. User refreshes web UI
11. UI confirms all components working
12. User can now connect device

### Component Detection
- Extension installed: Check for `window.chromeBridge`
- Native host installed: Call `ping` command
- Device connected: Call `listDevices` command

---

## 13. AI Agent Prompt Template

Use this template for each phase:

```
I am building a Feitian Security Key manager as a Chrome extension with a native Rust host.

Current Phase: [Phase Name]

Project Context:
- Monorepo with /web (React+Vite), /extension (MV3), /native (Rust)
- Native host communicates via JSON-RPC over stdin/stdout
- Extension bridges web UI to native host
- Only Feitian devices (VID 0x096e) are supported
- Single device operation at a time

Tasks for this phase:
1. [Task 1]
2. [Task 2]
3. [Task 3]

Technical Requirements:
- [Requirement 1]
- [Requirement 2]

Please provide:
1. Complete code implementation
2. Step-by-step instructions
3. Testing commands
4. Any manual steps I need to take

Generate production-ready code with proper error handling, logging, and documentation.
```

---

## 14. Development Checklist

### Phase 0 - Foundation
- [ ] Create repository structure
- [ ] Initialize all three subprojects
- [ ] Set up CI/CD pipeline
- [ ] Verify builds work

### Phase 1 - Native Host Core
- [ ] Implement JSON-RPC loop
- [ ] Implement device enumeration
- [ ] Test with real Feitian device

### Phase 2 - Extension Bridge
- [ ] Implement background worker
- [ ] Implement content script
- [ ] Test extension ↔ native host communication

### Phase 3 - Web UI Foundation
- [ ] Create React app structure
- [ ] Implement component detection
- [ ] Test device display

### Phase 4 - Transport Layer
- [ ] Implement HID transport
- [ ] Implement CCID transport
- [ ] Create debug console

### Phase 5 - Protocol Detection
- [ ] Implement detection logic
- [ ] Create protocol manager UI
- [ ] Test with multiple device models

### Phase 6-11 - Protocol Implementation
- [ ] FIDO2 complete
- [ ] U2F complete
- [ ] PIV complete
- [ ] OpenPGP complete
- [ ] OTP (HOTP) complete
- [ ] NDEF complete

### Phase 12 - Distribution
- [ ] Create installers
- [ ] Publish to Chrome Web Store
- [ ] Publish to Edge Add-ons
- [ ] Create documentation site

### Phase 13 - Security
- [ ] Input validation complete
- [ ] Security audit done
- [ ] Fuzzing tests pass

### Phase 14 - Testing & Docs
- [ ] All tests passing
- [ ] User docs complete
- [ ] Developer docs complete

---

## 15. Troubleshooting Guide

### Extension Not Connecting to Native Host
- Check native host is installed: `which feitian-sk-manager-host`
- Check manifest registration: Chrome/Edge should see the manifest file
- Check native host logs: Enable debug mode in native host

### Device Not Detected
- Verify device is Feitian (VID 0x096e)
- Check USB connection
- Try different USB port
- Check if device is locked by another process
- Verify permissions (udev rules on Linux)

### PIN Operations Failing
- Verify correct PIN entered
- Check retry counter (display only, no lockout)
- Try device reset if all retries exhausted

### Protocol Not Detected
- Device may not support that protocol
- Try firmware update
- Check device documentation for supported features

---

## 16. Future Enhancements (Post-Launch)

- TOTP support (in addition to HOTP)
- Backup/restore configuration
- Multi-device support
- Firmware update capability
- Advanced logging and diagnostics
- Plugin system for custom workflows
- Mobile companion app
- Cloud backup (encrypted)
- Team management features
- Batch operations
- Custom protocol extensions

---

## End of AGENTS.md

**Last Updated**: 2025-11-18  
**Version**: 2.0
**Maintainer**: [Rain Zhang]
