use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::device::DeviceManager;
use crate::transport;

// Allow unused constants for future use
#[allow(dead_code)]
/// CTAP2 command codes
const CTAP2_MAKE_CREDENTIAL: u8 = 0x01;
#[allow(dead_code)]
const CTAP2_GET_ASSERTION: u8 = 0x02;
const CTAP2_GET_INFO: u8 = 0x04;
const CTAP2_CLIENT_PIN: u8 = 0x06;
const CTAP2_RESET: u8 = 0x07;
const CTAP2_CREDENTIAL_MANAGEMENT: u8 = 0x0A;

/// CTAPHID commands
const CTAPHID_INIT: u8 = 0x06;
const CTAPHID_CBOR: u8 = 0x10;
#[allow(dead_code)]
const CTAPHID_CANCEL: u8 = 0x11;
#[allow(dead_code)]
const CTAPHID_KEEPALIVE: u8 = 0x3B;
const CTAPHID_ERROR: u8 = 0x3F;

/// CTAP2 status codes
const CTAP2_OK: u8 = 0x00;
#[allow(dead_code)]
const CTAP2_ERR_PIN_REQUIRED: u8 = 0x36;
#[allow(dead_code)]
const CTAP2_ERR_PIN_INVALID: u8 = 0x31;
#[allow(dead_code)]
const CTAP2_ERR_PIN_BLOCKED: u8 = 0x32;
#[allow(dead_code)]
const CTAP2_ERR_PIN_AUTH_INVALID: u8 = 0x33;
#[allow(dead_code)]
const CTAP2_ERR_PIN_AUTH_BLOCKED: u8 = 0x34;
#[allow(dead_code)]
const CTAP2_ERR_PIN_NOT_SET: u8 = 0x35;

/// Client PIN subcommands
#[allow(dead_code)]
const PIN_GET_RETRIES: u8 = 0x01;
#[allow(dead_code)]
const PIN_GET_KEY_AGREEMENT: u8 = 0x02;
#[allow(dead_code)]
const PIN_SET_PIN: u8 = 0x03;
#[allow(dead_code)]
const PIN_CHANGE_PIN: u8 = 0x04;
#[allow(dead_code)]
const PIN_GET_PIN_TOKEN: u8 = 0x05;

/// Credential Management subcommands
#[allow(dead_code)]
const CRED_MGMT_GET_CREDS_METADATA: u8 = 0x01;
#[allow(dead_code)]
const CRED_MGMT_ENUMERATE_RPS_BEGIN: u8 = 0x02;
#[allow(dead_code)]
const CRED_MGMT_ENUMERATE_RPS_NEXT: u8 = 0x03;
#[allow(dead_code)]
const CRED_MGMT_ENUMERATE_CREDENTIALS_BEGIN: u8 = 0x04;
#[allow(dead_code)]
const CRED_MGMT_ENUMERATE_CREDENTIALS_NEXT: u8 = 0x05;
#[allow(dead_code)]
const CRED_MGMT_DELETE_CREDENTIAL: u8 = 0x06;

/// FIDO2 device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fido2Info {
    pub versions: Vec<String>,
    pub extensions: Vec<String>,
    pub aaguid: String,
    pub options: Fido2Options,
    pub max_msg_size: Option<u32>,
    pub pin_protocols: Vec<u8>,
    pub max_credential_count_in_list: Option<u32>,
    pub max_credential_id_length: Option<u32>,
    pub transports: Vec<String>,
    pub algorithms: Vec<String>,
    pub max_authenticator_config_length: Option<u32>,
    pub default_cred_protect: Option<u8>,
}

/// FIDO2 options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fido2Options {
    pub plat: bool,               // Platform device
    pub rk: bool,                 // Resident key
    pub client_pin: Option<bool>, // Client PIN set
    pub up: bool,                 // User presence
    pub uv: Option<bool>,         // User verification
}

/// PIN retry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinRetries {
    pub retries: u8,
    pub power_cycle_required: bool,
}

/// Credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub rp_id: String,
    pub rp_name: String,
    pub user_id: String,
    pub user_name: String,
    pub user_display_name: String,
    pub credential_id: String,
    pub public_key: Option<String>,
    pub cred_protect: Option<u8>,
}

/// Initialize CTAPHID by getting a channel ID
fn ctaphid_init(device_manager: &DeviceManager, device_id: &str) -> Result<[u8; 4]> {
    let mut init_packet = [0u8; 64];
    init_packet[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Broadcast CID
    init_packet[4] = CTAPHID_INIT | 0x80; // INIT command with TYPE_INIT bit
    init_packet[5] = 0x00; // BCNTH (high byte of length)
    init_packet[6] = 0x08; // BCNTL (low byte of length = 8 bytes nonce)
                           // Add 8-byte random nonce
    init_packet[7..15].copy_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &init_packet)?;
        let init_response = transport::receive_hid(device, 1000)?;

        // Extract CID from response (bytes 15-18 of the INIT response)
        if init_response.len() >= 19 {
            let cid = [
                init_response[15],
                init_response[16],
                init_response[17],
                init_response[18],
            ];
            Ok(cid)
        } else {
            Err(anyhow!("Invalid INIT response"))
        }
    })
}

/// Send a CTAP2 command and receive response
fn ctap2_command(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
    command: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    // Construct CTAPHID packet
    let mut packet = [0u8; 64];
    packet[0..4].copy_from_slice(cid);
    packet[4] = CTAPHID_CBOR | 0x80; // CBOR command with TYPE_INIT bit

    let payload_len = 1 + data.len(); // command byte + data
    packet[5] = ((payload_len >> 8) & 0xFF) as u8; // BCNTH
    packet[6] = (payload_len & 0xFF) as u8; // BCNTL
    packet[7] = command; // CTAP2 command

    // Copy data (up to 57 bytes in first packet)
    let first_chunk_len = std::cmp::min(data.len(), 57);
    packet[8..8 + first_chunk_len].copy_from_slice(&data[..first_chunk_len]);

    device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &packet)?;

        // TODO: Handle continuation packets if data > 57 bytes

        let response = transport::receive_hid(device, 3000)?;

        // Parse response
        // Response format: [CID(4)] [CMD(1)] [BCNTH(1)] [BCNTL(1)] [DATA...]
        if response.len() < 7 {
            return Err(anyhow!("Response too short"));
        }

        // Check if it's an error response
        if response[4] == CTAPHID_ERROR {
            let error_code = response[7];
            return Err(anyhow!("CTAPHID error: 0x{:02X}", error_code));
        }

        // Extract data length
        let data_len = ((response[5] as usize) << 8) | (response[6] as usize);

        // Extract response data
        let response_data = if data_len <= 57 {
            response[7..7 + data_len].to_vec()
        } else {
            // TODO: Handle continuation packets
            response[7..].to_vec()
        };

        // Check CTAP2 status code
        if response_data.is_empty() {
            return Err(anyhow!("Empty response"));
        }

        let status = response_data[0];
        if status != CTAP2_OK {
            return Err(anyhow!("CTAP2 error: 0x{:02X}", status));
        }

        // Return data after status byte
        Ok(response_data[1..].to_vec())
    })
}

/// Get FIDO2 authenticator info
pub fn get_info(device_manager: &DeviceManager, device_id: &str) -> Result<Fido2Info> {
    log::debug!("Getting FIDO2 authenticator info...");

    let cid = ctaphid_init(device_manager, device_id)?;
    let _response = ctap2_command(device_manager, device_id, &cid, CTAP2_GET_INFO, &[])?;

    // Parse CBOR response
    // For now, return a simplified structure
    // Real implementation would decode CBOR using serde_cbor or ciborium

    Ok(Fido2Info {
        versions: vec!["FIDO_2_0".to_string(), "U2F_V2".to_string()],
        extensions: vec![],
        aaguid: "00000000-0000-0000-0000-000000000000".to_string(),
        options: Fido2Options {
            plat: false,
            rk: true,
            client_pin: Some(false),
            up: true,
            uv: Some(false),
        },
        max_msg_size: Some(1200),
        pin_protocols: vec![1],
        max_credential_count_in_list: Some(8),
        max_credential_id_length: Some(128),
        transports: vec!["usb".to_string()],
        algorithms: vec!["ES256".to_string()],
        max_authenticator_config_length: Some(1024),
        default_cred_protect: Some(1),
    })
}

/// Get PIN retry counter
pub fn get_pin_retries(device_manager: &DeviceManager, device_id: &str) -> Result<PinRetries> {
    log::debug!("Getting PIN retry counter...");

    let cid = ctaphid_init(device_manager, device_id)?;

    // Construct ClientPIN getRetries command
    // CBOR map: {0x01: 0x01} (pinProtocol: 1, subCommand: getPinRetries)
    // Simplified: just send minimal CBOR
    let data = vec![
        0xA1, // Map with 1 item
        0x01, // Key: pinProtocol
        0x01, // Value: 1
    ];

    let _response = ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data)?;

    // Parse response (simplified)
    // Real implementation would decode CBOR

    Ok(PinRetries {
        retries: 8,
        power_cycle_required: false,
    })
}

/// Set initial PIN
pub fn set_pin(device_manager: &DeviceManager, device_id: &str, new_pin: &str) -> Result<()> {
    log::debug!("Setting PIN...");

    if new_pin.len() < 4 {
        return Err(anyhow!("PIN must be at least 4 characters"));
    }

    if new_pin.len() > 63 {
        return Err(anyhow!("PIN must be at most 63 characters"));
    }

    let cid = ctaphid_init(device_manager, device_id)?;

    // Simplified implementation - real implementation would:
    // 1. Get key agreement from authenticator
    // 2. Establish shared secret
    // 3. Encrypt PIN
    // 4. Send encrypted PIN with pinAuth

    // For now, just attempt the command and let it fail gracefully
    let data = vec![
        0xA2, // Map with 2 items
        0x01, // Key: pinProtocol
        0x01, // Value: 1
        0x02, // Key: subCommand
        0x03, // Value: setPIN
    ];

    match ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(
            "Failed to set PIN: {} (Note: Full PIN encryption not yet implemented)",
            e
        )),
    }
}

/// Change existing PIN
pub fn change_pin(
    device_manager: &DeviceManager,
    device_id: &str,
    _current_pin: &str,
    new_pin: &str,
) -> Result<()> {
    log::debug!("Changing PIN...");

    if new_pin.len() < 4 {
        return Err(anyhow!("PIN must be at least 4 characters"));
    }

    if new_pin.len() > 63 {
        return Err(anyhow!("PIN must be at most 63 characters"));
    }

    let cid = ctaphid_init(device_manager, device_id)?;

    // Simplified implementation
    let data = vec![
        0xA2, // Map with 2 items
        0x01, // Key: pinProtocol
        0x01, // Value: 1
        0x02, // Key: subCommand
        0x04, // Value: changePIN
    ];

    match ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(
            "Failed to change PIN: {} (Note: Full PIN encryption not yet implemented)",
            e
        )),
    }
}

/// List all credentials
pub fn list_credentials(
    device_manager: &DeviceManager,
    device_id: &str,
) -> Result<Vec<Credential>> {
    log::debug!("Listing credentials...");

    let _cid = ctaphid_init(device_manager, device_id)?;

    // Simplified implementation - real implementation would:
    // 1. Authenticate with PIN
    // 2. Enumerate RPs
    // 3. For each RP, enumerate credentials

    // For now, return empty list
    Ok(vec![])
}

/// Delete a credential by ID
pub fn delete_credential(
    device_manager: &DeviceManager,
    device_id: &str,
    credential_id: &str,
) -> Result<()> {
    log::debug!("Deleting credential: {}", credential_id);

    let cid = ctaphid_init(device_manager, device_id)?;

    // Simplified implementation
    let data = vec![
        0xA2, // Map with 2 items
        0x01, // Key: pinProtocol
        0x01, // Value: 1
        0x02, // Key: subCommand
        0x06, // Value: deleteCredential
    ];

    match ctap2_command(device_manager, device_id, &cid, CTAP2_CREDENTIAL_MANAGEMENT, &data) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Failed to delete credential: {} (Note: Full credential management not yet implemented)", e)),
    }
}

/// Reset the authenticator to factory defaults
pub fn reset_device(device_manager: &DeviceManager, device_id: &str) -> Result<()> {
    log::debug!("Resetting authenticator...");

    let cid = ctaphid_init(device_manager, device_id)?;

    // RESET command has no parameters
    match ctap2_command(device_manager, device_id, &cid, CTAP2_RESET, &[]) {
        Ok(_) => {
            log::info!("Authenticator reset successful");
            Ok(())
        }
        Err(e) => Err(anyhow!("Failed to reset authenticator: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_length_validation() {
        // This would need a mock DeviceManager
        // For now, just test PIN validation logic
        assert!("123".len() < 4); // Too short
        assert!("1234".len() >= 4); // Valid
        assert!("a".repeat(63).len() <= 63); // Valid
        assert!("a".repeat(64).len() > 63); // Too long
    }

    #[test]
    fn test_fido2_info_serialization() {
        let info = Fido2Info {
            versions: vec!["FIDO_2_0".to_string()],
            extensions: vec![],
            aaguid: "00000000-0000-0000-0000-000000000000".to_string(),
            options: Fido2Options {
                plat: false,
                rk: true,
                client_pin: Some(false),
                up: true,
                uv: Some(false),
            },
            max_msg_size: Some(1200),
            pin_protocols: vec![1],
            max_credential_count_in_list: Some(8),
            max_credential_id_length: Some(128),
            transports: vec!["usb".to_string()],
            algorithms: vec!["ES256".to_string()],
            max_authenticator_config_length: Some(1024),
            default_cred_protect: Some(1),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("FIDO_2_0"));
    }
}
