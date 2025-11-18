# Native Messaging Host

This is the Rust-based native messaging host that provides the actual interface to Feitian security keys.

## Overview

The native host:
- Communicates with the Chrome extension via stdin/stdout using JSON-RPC protocol
- Provides device access via PC/SC (CCID) and HIDAPI
- Filters devices to only Feitian products (Vendor ID: 0x096e)
- Implements protocol handlers for FIDO2, U2F, PIV, OpenPGP, OTP, and NDEF

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run
```

## Testing

Test the native host standalone:

```bash
# Build first
cargo build

# Test ping command
echo '{"id":1,"command":"ping","params":{}}' | ./target/debug/feitian-sk-manager-native

# Test getVersion command
echo '{"id":2,"command":"getVersion","params":{}}' | ./target/debug/feitian-sk-manager-native
```

Expected output format:
```json
{"id":1,"status":"ok","result":{"message":"pong"}}
{"id":2,"status":"ok","result":{"version":"0.1.0","name":"feitian-sk-manager-native"}}
```

## Installation

The native host binary needs to be installed with a manifest file that Chrome can find.

### Manifest File

Create a file named `com.feitian.sk_manager.json` with the following content:

```json
{
  "name": "com.feitian.sk_manager",
  "description": "Feitian SK Manager Native Host",
  "path": "/path/to/feitian-sk-manager-native",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
```

### Installation Locations

**Linux:**
```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
cp com.feitian.sk_manager.json ~/.config/google-chrome/NativeMessagingHosts/
```

**macOS:**
```bash
mkdir -p ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts
cp com.feitian.sk_manager.json ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/
```

**Windows:**
Create a registry key at:
```
HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\com.feitian.sk_manager
```
With a default value pointing to the manifest JSON file.

## Protocol

### Request Format
```json
{
  "id": 1,
  "command": "commandName",
  "params": {
    "param1": "value1"
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
    "message": "Error description"
  }
}
```

## Implemented Commands

### Phase 0 (Current)
- `ping` - Health check (returns "pong")
- `getVersion` - Returns version information

### Future Phases
- `listDevices` - Enumerate Feitian devices
- `openDevice` - Open device connection
- `closeDevice` - Close device connection
- FIDO2/U2F commands
- PIV commands
- OpenPGP commands
- OTP commands
- NDEF commands

## Dependencies

- **serde/serde_json**: JSON serialization
- **tokio**: Async runtime (for future async operations)
- **anyhow**: Error handling
- **log/env_logger**: Logging

Future dependencies will include:
- **hidapi**: For HID device access (FIDO2, U2F, OTP)
- **pcsc**: For smart card access (PIV, OpenPGP)

## Logging

The native host logs to stderr. Set the `RUST_LOG` environment variable to control log level:

```bash
RUST_LOG=debug ./feitian-sk-manager-native
RUST_LOG=info ./feitian-sk-manager-native
RUST_LOG=error ./feitian-sk-manager-native
```

## Security

- All input is validated before processing
- Message length is limited to 1MB
- Only Feitian devices (VID 0x096e) are accessible
- No sensitive data is logged
- Memory is zeroed for sensitive operations
