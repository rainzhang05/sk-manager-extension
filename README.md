# Feitian SK Manager

A modern web-based management tool for Feitian Security Keys, supporting FIDO2, U2F, PIV, OpenPGP, OTP, and NDEF protocols.

## Architecture

This project uses a three-tier architecture:

```
Web UI (React + Vite + TypeScript)
        ⇅ window.postMessage / content script
Chrome Extension (Manifest V3)
        ⇅ chrome.runtime.connectNative()
Native Host (Rust binary, JSON-RPC)
        ⇅ PC/SC (CCID), HIDAPI
Feitian Security Key (Vendor ID: 0x096e)
```

## Project Structure

```
feitian-sk-manager/
├── .github/workflows/    # CI/CD pipelines
├── web/                  # React frontend (Vite + TypeScript)
├── extension/            # Chrome Extension (Manifest V3)
├── native/               # Rust native messaging host
└── docs/                 # Documentation
```

## Components

### Web UI (`/web`)
- **Tech Stack**: React 18, TypeScript, Vite
- **Design**: Black & white minimalist UI with 24px border radius
- **Features**: Device management, protocol configuration, certificate handling

### Chrome Extension (`/extension`)
- **Manifest**: V3
- **Purpose**: Bridge between web UI and native host
- **Permissions**: nativeMessaging, storage

### Native Host (`/native`)
- **Language**: Rust
- **Protocol**: JSON-RPC over stdin/stdout
- **Libraries**: pcsc (CCID), hidapi (FIDO/OTP), serde_json, tokio

## Development Setup

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Chrome or Edge browser

### Building

#### Web UI
```bash
cd web
npm install
npm run dev        # Start dev server
npm run build      # Production build
```

#### Native Host
```bash
cd native
cargo build        # Debug build
cargo build --release  # Production build
cargo test         # Run tests
```

#### Chrome Extension
```bash
cd extension
# Load unpacked extension in Chrome:
# 1. Navigate to chrome://extensions/
# 2. Enable "Developer mode"
# 3. Click "Load unpacked"
# 4. Select the extension/ directory
```

### Running Tests

```bash
# All components
npm test           # From root directory

# Individual components
cd web && npm test
cd native && cargo test
```

## Installation

### For Users

1. Install the Chrome Extension from the [Chrome Web Store](#) (coming soon)
2. Download and install the native host for your platform:
   - **Windows**: [Download .exe installer](#)
   - **macOS**: [Download .pkg](#)
   - **Linux**: [Download .deb or .rpm](#)

### For Developers

See [Development Setup](#development-setup) above.

## Supported Devices

This application supports Feitian security keys with Vendor ID `0x096e`, including:
- ePass FIDO (PID: 0x0850)
- ePass FIDO-NFC (PID: 0x0852)
- BioPass FIDO (PID: 0x0853)
- AllinPass FIDO (PID: 0x0854)
- ePass K9 FIDO (PID: 0x0856)

## Protocols Supported

- **FIDO2 (CTAP2)**: PIN management, credential management, device reset
- **U2F (CTAP1)**: Registration and authentication
- **PIV**: Certificate management, key generation, PIN/PUK management
- **OpenPGP**: Key import/export, card data management
- **OTP**: HOTP configuration (TOTP coming soon)
- **NDEF**: NFC data read/write

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Security

For security concerns, please email security@example.com.

## Acknowledgments

- Inspired by YubiKey Manager and similar tools
- Built with modern web technologies
- Designed for security and usability