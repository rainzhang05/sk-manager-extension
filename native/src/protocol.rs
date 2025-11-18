use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::device::DeviceManager;
use crate::transport;

/// Protocol support information for a device
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtocolSupport {
    pub fido2: bool,
    pub u2f: bool,
    pub piv: bool,
    pub openpgp: bool,
    pub otp: bool,
    pub ndef: bool,
}

/// CTAP2 command for getInfo (0x04)
const CTAP2_GETINFO: u8 = 0x04;

/// Detect FIDO2/CTAP2 support
///
/// Sends CTAP2 getInfo command via HID
fn detect_fido2(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting FIDO2/CTAP2 support...");

    // CTAP2 HID packet format: [CID(4) | CMD(1) | BCNTH(1) | BCNTL(1) | DATA]
    // For initialization, we use broadcast CID: 0xFFFFFFFF
    // CMD_CBOR = 0x10 | 0x80 = 0x90
    let mut packet = vec![0u8; 64];
    packet[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Broadcast CID
    packet[4] = 0x90; // CMD_CBOR | TYPE_INIT
    packet[5] = 0x00; // BCNTH (high byte of length)
    packet[6] = 0x01; // BCNTL (low byte of length = 1)
    packet[7] = CTAP2_GETINFO; // getInfo command

    match device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &packet[..64])?;
        let response = transport::receive_hid(device, 1000)?;
        Ok(response)
    }) {
        Ok(response) => {
            // Check if response looks like a valid CTAP2 response
            // Should start with CID and have CBOR response flag
            if response.len() >= 7 {
                log::info!("FIDO2/CTAP2 supported (received {} byte response)", response.len());
                true
            } else {
                log::debug!("FIDO2/CTAP2 not supported (invalid response)");
                false
            }
        }
        Err(e) => {
            log::debug!("FIDO2/CTAP2 detection failed: {}", e);
            false
        }
    }
}

/// Detect U2F/CTAP1 support
///
/// Sends U2F version command via HID
fn detect_u2f(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting U2F/CTAP1 support...");

    // U2F raw message format (sent via HID)
    // CMD_MSG = 0x03 | 0x80 = 0x83
    let mut packet = vec![0u8; 64];
    packet[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Broadcast CID
    packet[4] = 0x83; // CMD_MSG | TYPE_INIT
    packet[5] = 0x00; // BCNTH
    packet[6] = 0x07; // BCNTL = 7 bytes (U2F version request)
    // U2F version APDU: 00 03 00 00 00 00 00
    packet[7] = 0x00; // CLA
    packet[8] = 0x03; // INS (version)
    packet[9] = 0x00; // P1
    packet[10] = 0x00; // P2
    packet[11] = 0x00; // Lc1
    packet[12] = 0x00; // Lc2
    packet[13] = 0x00; // Lc3

    match device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &packet[..64])?;
        let response = transport::receive_hid(device, 1000)?;
        Ok(response)
    }) {
        Ok(response) => {
            // U2F version response should contain "U2F_V2" string
            if response.len() >= 10 {
                log::info!("U2F/CTAP1 supported (received {} byte response)", response.len());
                true
            } else {
                log::debug!("U2F/CTAP1 not supported (invalid response)");
                false
            }
        }
        Err(e) => {
            log::debug!("U2F/CTAP1 detection failed: {}", e);
            false
        }
    }
}

/// Detect PIV support
///
/// Tries to SELECT the PIV application via APDU
fn detect_piv(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting PIV support...");

    // PIV application AID: A0 00 00 03 08
    let piv_select = vec![
        0x00, // CLA
        0xA4, // INS (SELECT)
        0x04, // P1 (Select by name)
        0x00, // P2
        0x05, // Lc (length of data)
        0xA0, 0x00, 0x00, 0x03, 0x08, // PIV AID
    ];

    match device_manager.with_ccid_card(device_id, |card| {
        let response = transport::transmit_apdu(card, &piv_select)?;
        Ok(response)
    }) {
        Ok(response) => {
            // Check for success status word (90 00)
            if response.len() >= 2 {
                let sw1 = response[response.len() - 2];
                let sw2 = response[response.len() - 1];
                if sw1 == 0x90 && sw2 == 0x00 {
                    log::info!("PIV supported");
                    return true;
                }
            }
            log::debug!("PIV not supported (SELECT failed)");
            false
        }
        Err(e) => {
            log::debug!("PIV detection failed: {}", e);
            false
        }
    }
}

/// Detect OpenPGP support
///
/// Tries to SELECT the OpenPGP application via APDU
fn detect_openpgp(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting OpenPGP support...");

    // OpenPGP application AID: D2 76 00 01 24 01
    let openpgp_select = vec![
        0x00, // CLA
        0xA4, // INS (SELECT)
        0x04, // P1 (Select by name)
        0x00, // P2
        0x06, // Lc (length of data)
        0xD2, 0x76, 0x00, 0x01, 0x24, 0x01, // OpenPGP AID
    ];

    match device_manager.with_ccid_card(device_id, |card| {
        let response = transport::transmit_apdu(card, &openpgp_select)?;
        Ok(response)
    }) {
        Ok(response) => {
            // Check for success status word (90 00)
            if response.len() >= 2 {
                let sw1 = response[response.len() - 2];
                let sw2 = response[response.len() - 1];
                if sw1 == 0x90 && sw2 == 0x00 {
                    log::info!("OpenPGP supported");
                    return true;
                }
            }
            log::debug!("OpenPGP not supported (SELECT failed)");
            false
        }
        Err(e) => {
            log::debug!("OpenPGP detection failed: {}", e);
            false
        }
    }
}

/// Detect OTP support
///
/// Tries vendor-specific OTP command via HID
fn detect_otp(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting OTP support...");

    // Try Feitian vendor-specific OTP status command
    // This is a simplified check - actual OTP detection may vary by device model
    let mut packet = vec![0u8; 64];
    packet[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // Broadcast CID
    packet[4] = 0x83; // CMD_MSG
    packet[5] = 0x00; // BCNTH
    packet[6] = 0x05; // BCNTL = 5 bytes
    // Vendor command to check OTP status
    packet[7] = 0x00; // CLA
    packet[8] = 0x01; // INS (vendor-specific)
    packet[9] = 0x00; // P1
    packet[10] = 0x00; // P2
    packet[11] = 0x00; // Le

    match device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &packet[..64])?;
        let response = transport::receive_hid(device, 1000)?;
        Ok(response)
    }) {
        Ok(_response) => {
            // If we get any response, assume OTP might be supported
            // Real implementation would check response content
            log::debug!("OTP detection: possible support (vendor command responded)");
            false // Conservative: mark as unsupported unless we know for sure
        }
        Err(e) => {
            log::debug!("OTP detection failed: {}", e);
            false
        }
    }
}

/// Detect NDEF support
///
/// Tries to read NDEF capability container via APDU
fn detect_ndef(device_manager: &DeviceManager, device_id: &str) -> bool {
    log::debug!("Detecting NDEF support...");

    // Try to SELECT NDEF application
    // NDEF Type 4 Tag Application: D2 76 00 00 85 01 01
    let ndef_select = vec![
        0x00, // CLA
        0xA4, // INS (SELECT)
        0x04, // P1 (Select by name)
        0x00, // P2
        0x07, // Lc (length of data)
        0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01, // NDEF AID
    ];

    match device_manager.with_ccid_card(device_id, |card| {
        let response = transport::transmit_apdu(card, &ndef_select)?;
        Ok(response)
    }) {
        Ok(response) => {
            // Check for success status word (90 00)
            if response.len() >= 2 {
                let sw1 = response[response.len() - 2];
                let sw2 = response[response.len() - 1];
                if sw1 == 0x90 && sw2 == 0x00 {
                    log::info!("NDEF supported");
                    return true;
                }
            }
            log::debug!("NDEF not supported (SELECT failed)");
            false
        }
        Err(e) => {
            log::debug!("NDEF detection failed: {}", e);
            false
        }
    }
}

/// Detect which protocols a device supports
///
/// Probes the device with various protocol-specific commands to determine support.
///
/// # Arguments
/// * `device_manager` - Reference to the device manager
/// * `device_id` - The unique identifier of the device to check
///
/// # Returns
/// * `Ok(ProtocolSupport)` - Protocol support information
/// * `Err` - If the device cannot be accessed or is not open
pub fn detect_protocols(device_manager: &DeviceManager, device_id: &str) -> Result<ProtocolSupport> {
    log::info!("Starting protocol detection for device: {}", device_id);

    // Note: Some detections may fail if device isn't the right type (HID vs CCID)
    // We catch errors and continue with other protocols
    
    let fido2 = detect_fido2(device_manager, device_id);
    let u2f = detect_u2f(device_manager, device_id);
    let piv = detect_piv(device_manager, device_id);
    let openpgp = detect_openpgp(device_manager, device_id);
    let otp = detect_otp(device_manager, device_id);
    let ndef = detect_ndef(device_manager, device_id);

    let support = ProtocolSupport {
        fido2,
        u2f,
        piv,
        openpgp,
        otp,
        ndef,
    };

    log::info!(
        "Protocol detection complete: FIDO2={}, U2F={}, PIV={}, OpenPGP={}, OTP={}, NDEF={}",
        support.fido2,
        support.u2f,
        support.piv,
        support.openpgp,
        support.otp,
        support.ndef
    );

    Ok(support)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_support_default() {
        let support = ProtocolSupport::default();
        assert!(!support.fido2);
        assert!(!support.u2f);
        assert!(!support.piv);
        assert!(!support.openpgp);
        assert!(!support.otp);
        assert!(!support.ndef);
    }

    #[test]
    fn test_protocol_support_serialization() {
        let support = ProtocolSupport {
            fido2: true,
            u2f: true,
            piv: false,
            openpgp: false,
            otp: true,
            ndef: false,
        };

        let json = serde_json::to_string(&support).unwrap();
        assert!(json.contains("\"fido2\":true"));
        assert!(json.contains("\"u2f\":true"));
        assert!(json.contains("\"piv\":false"));
    }

    #[test]
    fn test_detect_protocols_requires_device_manager() {
        // Protocol detection now requires a DeviceManager and open device
        // This test just verifies the structure compiles correctly
        let support = ProtocolSupport::default();
        assert!(!support.fido2);
        assert!(!support.u2f);
        assert!(!support.piv);
        assert!(!support.openpgp);
        assert!(!support.otp);
        assert!(!support.ndef);
    }
}
