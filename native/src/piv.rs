use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::device::DeviceManager;
use crate::transport;

// PIV Application AID
const PIV_AID: [u8; 5] = [0xA0, 0x00, 0x00, 0x03, 0x08];

// PIV Data Object Tags
const TAG_CHUID: [u8; 3] = [0x5F, 0xC1, 0x02]; // Card Holder Unique Identifier
const TAG_CERT_PIV_AUTH: [u8; 3] = [0x5F, 0xC1, 0x05]; // X.509 Certificate for PIV Authentication
const TAG_CERT_CARD_AUTH: [u8; 3] = [0x5F, 0xC1, 0x01]; // X.509 Certificate for Card Authentication
const TAG_CERT_DIGITAL_SIG: [u8; 3] = [0x5F, 0xC1, 0x0A]; // X.509 Certificate for Digital Signature
const TAG_CERT_KEY_MGMT: [u8; 3] = [0x5F, 0xC1, 0x0B]; // X.509 Certificate for Key Management
const TAG_PRINTED_INFO: [u8; 3] = [0x5F, 0xC1, 0x09]; // Printed Information
const TAG_FACIAL_IMAGE: [u8; 3] = [0x5F, 0xC1, 0x08]; // Cardholder Facial Image
const TAG_DISCOVERY: [u8; 1] = [0x7E]; // Discovery Object

// INS byte for PIV commands
const INS_SELECT: u8 = 0xA4;
const INS_GET_DATA: u8 = 0xCB;
const INS_VERIFY: u8 = 0x20;
const INS_GET_RESPONSE: u8 = 0xC0;

/// PIV device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivInfo {
    pub selected: bool,
    pub chuid: Option<String>,
    pub discovery: Option<PivDiscovery>,
    pub certificates: Vec<PivCertificate>,
}

/// PIV Discovery Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivDiscovery {
    pub piv_card_application_aid: Option<String>,
    pub pin_usage_policy: Option<String>,
}

/// PIV Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivCertificate {
    pub slot: String,
    pub slot_name: String,
    pub present: bool,
    pub certificate_data: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub serial_number: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
}

/// APDU command result for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApduLog {
    pub command: String,
    pub command_hex: String,
    pub response_hex: String,
    pub sw1: u8,
    pub sw2: u8,
    pub status: String,
    pub description: String,
}

/// PIV data retrieval result with activity logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivDataResult {
    pub info: PivInfo,
    pub activity_log: Vec<ApduLog>,
}

/// Format bytes as hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

/// Parse status word to human-readable description
fn status_word_description(sw1: u8, sw2: u8) -> String {
    match (sw1, sw2) {
        (0x90, 0x00) => "Success".to_string(),
        (0x61, _) => format!("{} bytes of response data available", sw2),
        (0x62, 0x81) => "Part of returned data may be corrupted".to_string(),
        (0x62, 0x82) => "End of file reached before reading Le bytes".to_string(),
        (0x63, 0x00) => "Verification failed".to_string(),
        (0x63, n) if n >= 0xC0 => format!("Verification failed, {} retries remaining", n & 0x0F),
        (0x64, _) => "Execution error".to_string(),
        (0x65, _) => "Memory failure".to_string(),
        (0x67, 0x00) => "Wrong length".to_string(),
        (0x68, 0x81) => "Logical channel not supported".to_string(),
        (0x68, 0x82) => "Secure messaging not supported".to_string(),
        (0x69, 0x81) => "Command incompatible with file structure".to_string(),
        (0x69, 0x82) => "Security status not satisfied".to_string(),
        (0x69, 0x83) => "Authentication method blocked".to_string(),
        (0x69, 0x84) => "Referenced data invalidated".to_string(),
        (0x69, 0x85) => "Conditions of use not satisfied".to_string(),
        (0x69, 0x86) => "Command not allowed (no current EF)".to_string(),
        (0x6A, 0x80) => "Incorrect parameters in data field".to_string(),
        (0x6A, 0x81) => "Function not supported".to_string(),
        (0x6A, 0x82) => "File not found / Data object not found".to_string(),
        (0x6A, 0x83) => "Record not found".to_string(),
        (0x6A, 0x84) => "Not enough memory space".to_string(),
        (0x6A, 0x86) => "Incorrect parameters P1-P2".to_string(),
        (0x6A, 0x88) => "Referenced data not found".to_string(),
        (0x6B, 0x00) => "Wrong parameter(s) P1-P2".to_string(),
        (0x6C, n) => format!("Wrong Le field; {} bytes available", n),
        (0x6D, 0x00) => "Instruction code not supported or invalid".to_string(),
        (0x6E, 0x00) => "Class not supported".to_string(),
        (0x6F, 0x00) => "No precise diagnosis".to_string(),
        _ => format!("Unknown status: {:02X} {:02X}", sw1, sw2),
    }
}

/// Build SELECT APDU command
fn build_select_apdu(aid: &[u8]) -> Vec<u8> {
    let mut apdu = vec![
        0x00, // CLA
        INS_SELECT, // INS
        0x04, // P1 = Select by name
        0x00, // P2 = First or only occurrence
        aid.len() as u8, // Lc
    ];
    apdu.extend_from_slice(aid);
    apdu
}

/// Build GET DATA APDU command
fn build_get_data_apdu(tag: &[u8]) -> Vec<u8> {
    // GET DATA: 00 CB 3F FF [Lc] 5C [tag length] [tag] 00
    let mut data = vec![0x5C, tag.len() as u8];
    data.extend_from_slice(tag);

    let mut apdu = vec![
        0x00, // CLA
        INS_GET_DATA, // INS
        0x3F, // P1
        0xFF, // P2
        data.len() as u8, // Lc
    ];
    apdu.extend_from_slice(&data);
    apdu.push(0x00); // Le = 0 (maximum response)
    apdu
}

/// Build GET RESPONSE APDU
fn build_get_response_apdu(le: u8) -> Vec<u8> {
    vec![
        0x00, // CLA
        INS_GET_RESPONSE, // INS
        0x00, // P1
        0x00, // P2
        le, // Le
    ]
}

/// Transmit APDU and handle response chaining (61 XX)
fn transmit_apdu_with_chaining(
    device_manager: &DeviceManager,
    device_id: &str,
    apdu: &[u8],
    command_name: &str,
    activity_log: &mut Vec<ApduLog>,
) -> Result<Vec<u8>> {
    log::debug!("Transmitting APDU: {} - {}", command_name, bytes_to_hex(apdu));

    // Add timeout for device operations
    let response = device_manager.with_ccid_card(device_id, |card| {
        transport::transmit_apdu(card, apdu)
    }).map_err(|e| {
        log::error!("Failed to transmit APDU to device {}: {}", device_id, e);
        e
    })?;

    if response.len() < 2 {
        return Err(anyhow!("Response too short"));
    }

    let sw1 = response[response.len() - 2];
    let sw2 = response[response.len() - 1];
    let data = response[..response.len() - 2].to_vec();

    // Log the initial command
    activity_log.push(ApduLog {
        command: command_name.to_string(),
        command_hex: bytes_to_hex(apdu),
        response_hex: bytes_to_hex(&response),
        sw1,
        sw2,
        status: if sw1 == 0x90 && sw2 == 0x00 { "OK".to_string() }
                else if sw1 == 0x61 { "MORE_DATA".to_string() }
                else { "ERROR".to_string() },
        description: status_word_description(sw1, sw2),
    });

    // Handle response chaining (61 XX = more data available)
    if sw1 == 0x61 {
        let mut full_response = data;
        let mut remaining = sw2;

        loop {
            let get_response = build_get_response_apdu(remaining);
            log::debug!("GET RESPONSE: {}", bytes_to_hex(&get_response));

            let chunk = device_manager.with_ccid_card(device_id, |card| {
                transport::transmit_apdu(card, &get_response)
            }).map_err(|e| {
                log::error!("Failed to transmit GET RESPONSE to device {}: {}", device_id, e);
                e
            })?;

            if chunk.len() < 2 {
                return Err(anyhow!("GET RESPONSE too short"));
            }

            let chunk_sw1 = chunk[chunk.len() - 2];
            let chunk_sw2 = chunk[chunk.len() - 1];
            let chunk_data = &chunk[..chunk.len() - 2];

            activity_log.push(ApduLog {
                command: format!("{} (GET RESPONSE)", command_name),
                command_hex: bytes_to_hex(&get_response),
                response_hex: bytes_to_hex(&chunk),
                sw1: chunk_sw1,
                sw2: chunk_sw2,
                status: if chunk_sw1 == 0x90 && chunk_sw2 == 0x00 { "OK".to_string() }
                        else if chunk_sw1 == 0x61 { "MORE_DATA".to_string() }
                        else { "ERROR".to_string() },
                description: status_word_description(chunk_sw1, chunk_sw2),
            });

            full_response.extend_from_slice(chunk_data);

            if chunk_sw1 == 0x90 && chunk_sw2 == 0x00 {
                break;
            } else if chunk_sw1 == 0x61 {
                remaining = chunk_sw2;
            } else {
                return Err(anyhow!("GET RESPONSE error: {:02X} {:02X}", chunk_sw1, chunk_sw2));
            }
        }

        return Ok(full_response);
    }

    // Check for error status
    if sw1 != 0x90 || sw2 != 0x00 {
        // 6A 82 = file not found is acceptable (means certificate not present)
        if sw1 == 0x6A && sw2 == 0x82 {
            return Ok(vec![]); // Return empty response
        }
        return Err(anyhow!("APDU error: {:02X} {:02X} - {}", sw1, sw2, status_word_description(sw1, sw2)));
    }

    Ok(data)
}

/// Parse TLV (Tag-Length-Value) data
fn parse_tlv(data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Parse tag
        let mut tag = vec![data[i]];
        i += 1;

        // Multi-byte tag (if lower 5 bits are all 1s)
        if tag[0] & 0x1F == 0x1F {
            while i < data.len() && (data[i] & 0x80) != 0 {
                tag.push(data[i]);
                i += 1;
            }
            if i < data.len() {
                tag.push(data[i]);
                i += 1;
            }
        }

        if i >= data.len() {
            break;
        }

        // Parse length
        let mut length = data[i] as usize;
        i += 1;

        if length == 0x81 && i < data.len() {
            length = data[i] as usize;
            i += 1;
        } else if length == 0x82 && i + 1 < data.len() {
            length = ((data[i] as usize) << 8) | (data[i + 1] as usize);
            i += 2;
        } else if length == 0x83 && i + 2 < data.len() {
            length = ((data[i] as usize) << 16) | ((data[i + 1] as usize) << 8) | (data[i + 2] as usize);
            i += 3;
        }

        // Parse value
        if i + length <= data.len() {
            let value = data[i..i + length].to_vec();
            result.push((tag, value));
            i += length;
        } else {
            break;
        }
    }

    result
}

/// Extract certificate from PIV data object
fn extract_certificate_from_data(data: &[u8]) -> Option<Vec<u8>> {
    let tlv = parse_tlv(data);

    // Look for tag 0x53 (data field containing certificate)
    for (tag, value) in &tlv {
        if tag == &[0x53] {
            // Parse inner TLV
            let inner_tlv = parse_tlv(value);
            for (inner_tag, inner_value) in inner_tlv {
                // Tag 0x70 contains the certificate
                if inner_tag == vec![0x70] {
                    return Some(inner_value);
                }
            }
        }
    }

    None
}

/// Get PIV information from the device
pub fn get_piv_data(device_manager: &DeviceManager, device_id: &str) -> Result<PivDataResult> {
    log::info!("Getting PIV data from device: {}", device_id);

    let mut activity_log = Vec::new();
    let mut info = PivInfo {
        selected: false,
        chuid: None,
        discovery: None,
        certificates: Vec::new(),
    };

    // Step 1: SELECT PIV application
    let select_apdu = build_select_apdu(&PIV_AID);
    let select_response = transmit_apdu_with_chaining(
        device_manager,
        device_id,
        &select_apdu,
        "SELECT PIV Application",
        &mut activity_log
    )?;

    if !select_response.is_empty() || activity_log.last().map(|l| l.sw1 == 0x90 && l.sw2 == 0x00).unwrap_or(false) {
        info.selected = true;
        log::info!("PIV application selected successfully");
    } else {
        return Err(anyhow!("Failed to select PIV application"));
    }

    // Step 2: Get Discovery Object
    log::debug!("Getting Discovery Object...");
    let discovery_apdu = build_get_data_apdu(&TAG_DISCOVERY);
    match transmit_apdu_with_chaining(
        device_manager,
        device_id,
        &discovery_apdu,
        "GET DATA (Discovery Object)",
        &mut activity_log
    ) {
        Ok(data) if !data.is_empty() => {
            let tlv = parse_tlv(&data);
            let mut discovery = PivDiscovery {
                piv_card_application_aid: None,
                pin_usage_policy: None,
            };

            for (tag, value) in tlv {
                if tag == vec![0x7E] {
                    let inner_tlv = parse_tlv(&value);
                    for (inner_tag, inner_value) in inner_tlv {
                        match inner_tag.as_slice() {
                            [0x4F] => {
                                discovery.piv_card_application_aid = Some(bytes_to_hex(&inner_value));
                            }
                            [0x5F, 0x2F] => {
                                discovery.pin_usage_policy = Some(bytes_to_hex(&inner_value));
                            }
                            _ => {}
                        }
                    }
                }
            }

            info.discovery = Some(discovery);
        }
        Ok(_) => {
            log::debug!("Discovery object is empty or not present");
        }
        Err(e) => {
            log::warn!("Failed to get discovery object: {}", e);
        }
    }

    // Step 3: Get CHUID
    log::debug!("Getting CHUID...");
    let chuid_apdu = build_get_data_apdu(&TAG_CHUID);
    match transmit_apdu_with_chaining(
        device_manager,
        device_id,
        &chuid_apdu,
        "GET DATA (CHUID)",
        &mut activity_log
    ) {
        Ok(data) if !data.is_empty() => {
            // Parse CHUID TLV to extract GUID
            let tlv = parse_tlv(&data);
            for (tag, value) in tlv {
                if tag == vec![0x53] {
                    let inner_tlv = parse_tlv(&value);
                    for (inner_tag, inner_value) in inner_tlv {
                        // Tag 0x34 is the GUID
                        if inner_tag == vec![0x34] && inner_value.len() == 16 {
                            info.chuid = Some(format!(
                                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                                inner_value[0], inner_value[1], inner_value[2], inner_value[3],
                                inner_value[4], inner_value[5],
                                inner_value[6], inner_value[7],
                                inner_value[8], inner_value[9],
                                inner_value[10], inner_value[11], inner_value[12], inner_value[13], inner_value[14], inner_value[15]
                            ));
                        }
                    }
                }
            }

            if info.chuid.is_none() {
                // Fallback: just use hex of the whole data
                info.chuid = Some(bytes_to_hex(&data));
            }
        }
        Ok(_) => {
            log::debug!("CHUID is empty or not present");
        }
        Err(e) => {
            log::warn!("Failed to get CHUID: {}", e);
        }
    }

    // Step 4: Check for certificates in each slot
    let cert_slots = [
        ("9A", "PIV Authentication", TAG_CERT_PIV_AUTH.as_slice()),
        ("9E", "Card Authentication", TAG_CERT_CARD_AUTH.as_slice()),
        ("9C", "Digital Signature", TAG_CERT_DIGITAL_SIG.as_slice()),
        ("9D", "Key Management", TAG_CERT_KEY_MGMT.as_slice()),
    ];

    for (slot, slot_name, tag) in cert_slots {
        log::debug!("Checking certificate in slot {} ({})...", slot, slot_name);

        let cert_apdu = build_get_data_apdu(tag);
        match transmit_apdu_with_chaining(
            device_manager,
            device_id,
            &cert_apdu,
            &format!("GET DATA (Certificate {})", slot),
            &mut activity_log
        ) {
            Ok(data) if !data.is_empty() => {
                let cert_data = extract_certificate_from_data(&data);

                info.certificates.push(PivCertificate {
                    slot: slot.to_string(),
                    slot_name: slot_name.to_string(),
                    present: cert_data.is_some(),
                    certificate_data: cert_data.as_ref().map(|c| bytes_to_hex(c)),
                    subject: None, // Would need X.509 parsing
                    issuer: None,
                    serial_number: None,
                    not_before: None,
                    not_after: None,
                });
            }
            Ok(_) => {
                info.certificates.push(PivCertificate {
                    slot: slot.to_string(),
                    slot_name: slot_name.to_string(),
                    present: false,
                    certificate_data: None,
                    subject: None,
                    issuer: None,
                    serial_number: None,
                    not_before: None,
                    not_after: None,
                });
            }
            Err(e) => {
                log::debug!("Certificate {} not present or error: {}", slot, e);
                info.certificates.push(PivCertificate {
                    slot: slot.to_string(),
                    slot_name: slot_name.to_string(),
                    present: false,
                    certificate_data: None,
                    subject: None,
                    issuer: None,
                    serial_number: None,
                    not_before: None,
                    not_after: None,
                });
            }
        }
    }

    log::info!("PIV data retrieval complete. {} APDU commands executed.", activity_log.len());

    Ok(PivDataResult {
        info,
        activity_log,
    })
}

/// Select PIV application (simple test command)
pub fn select_piv(device_manager: &DeviceManager, device_id: &str) -> Result<bool> {
    log::debug!("Selecting PIV application...");

    let mut activity_log = Vec::new();
    let select_apdu = build_select_apdu(&PIV_AID);

    transmit_apdu_with_chaining(
        device_manager,
        device_id,
        &select_apdu,
        "SELECT PIV Application",
        &mut activity_log
    )?;

    // Check if the last command was successful
    let success = activity_log.last()
        .map(|log| log.sw1 == 0x90 && log.sw2 == 0x00)
        .unwrap_or(false);

    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_select_apdu() {
        let apdu = build_select_apdu(&PIV_AID);
        assert_eq!(apdu, vec![0x00, 0xA4, 0x04, 0x00, 0x05, 0xA0, 0x00, 0x00, 0x03, 0x08]);
    }

    #[test]
    fn test_build_get_data_apdu() {
        let apdu = build_get_data_apdu(&TAG_CHUID);
        // 00 CB 3F FF 05 5C 03 5F C1 02 00
        assert_eq!(apdu.len(), 11);
        assert_eq!(apdu[0], 0x00); // CLA
        assert_eq!(apdu[1], 0xCB); // INS
        assert_eq!(apdu[2], 0x3F); // P1
        assert_eq!(apdu[3], 0xFF); // P2
    }

    #[test]
    fn test_status_word_description() {
        assert_eq!(status_word_description(0x90, 0x00), "Success");
        assert_eq!(status_word_description(0x6A, 0x82), "File not found / Data object not found");
        assert!(status_word_description(0x63, 0xC3).contains("3 retries"));
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0x00, 0xA4, 0x04]), "00 A4 04");
    }

    #[test]
    fn test_parse_tlv_simple() {
        let data = vec![0x53, 0x03, 0x01, 0x02, 0x03];
        let result = parse_tlv(&data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, vec![0x53]);
        assert_eq!(result[0].1, vec![0x01, 0x02, 0x03]);
    }
}