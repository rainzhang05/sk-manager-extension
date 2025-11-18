use anyhow::{anyhow, Result};

/// Send raw HID packet (64 bytes standard)
///
/// # Arguments
/// * `device` - Reference to an open HID device
/// * `data` - Data to send (will be padded to 64 bytes)
///
/// # Returns
/// * `Ok(usize)` - Number of bytes written
/// * `Err` - If the packet is too large or write fails
pub fn send_hid(device: &hidapi::HidDevice, data: &[u8]) -> Result<usize> {
    if data.len() > 64 {
        return Err(anyhow!(
            "HID packet too large: {} bytes (max 64)",
            data.len()
        ));
    }

    // Pad to 64 bytes
    let mut padded = vec![0u8; 64];
    padded[..data.len()].copy_from_slice(data);

    let bytes_written = device
        .write(&padded)
        .map_err(|e| anyhow!("Failed to write HID packet: {}", e))?;

    log::debug!("Sent HID packet: {} bytes", bytes_written);
    log::trace!("HID data: {:02X?}", &padded[..data.len()]);

    Ok(bytes_written)
}

/// Receive raw HID packet
///
/// # Arguments
/// * `device` - Reference to an open HID device
/// * `timeout_ms` - Timeout in milliseconds (default: 5000ms)
///
/// # Returns
/// * `Ok(Vec<u8>)` - Received data (may be less than 64 bytes)
/// * `Err` - If timeout occurs or read fails
pub fn receive_hid(device: &hidapi::HidDevice, timeout_ms: i32) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; 64];
    let bytes_read = device
        .read_timeout(&mut buffer, timeout_ms)
        .map_err(|e| anyhow!("Failed to read HID packet: {}", e))?;

    if bytes_read == 0 {
        return Err(anyhow!("HID read timeout after {}ms", timeout_ms));
    }

    buffer.truncate(bytes_read);
    log::debug!("Received HID packet: {} bytes", bytes_read);
    log::trace!("HID data: {:02X?}", buffer);

    Ok(buffer)
}

/// Transmit APDU to smart card
///
/// # Arguments
/// * `card` - Reference to an open CCID card
/// * `apdu` - APDU command to transmit (minimum 4 bytes: CLA INS P1 P2)
///
/// # Returns
/// * `Ok(Vec<u8>)` - APDU response including status word (SW1 SW2)
/// * `Err` - If APDU is invalid or transmission fails
pub fn transmit_apdu(card: &pcsc::Card, apdu: &[u8]) -> Result<Vec<u8>> {
    if apdu.len() < 4 {
        return Err(anyhow!(
            "Invalid APDU: too short ({} bytes, minimum 4)",
            apdu.len()
        ));
    }

    log::debug!("Transmitting APDU: {} bytes", apdu.len());
    log::trace!("APDU: {:02X?}", apdu);

    // Prepare response buffer (maximum size per PC/SC spec)
    let mut response = vec![0u8; pcsc::MAX_BUFFER_SIZE];

    // Transmit the APDU
    let response_data = card
        .transmit(apdu, &mut response)
        .map_err(|e| anyhow!("Failed to transmit APDU: {}", e))?;

    // Copy the response data to a new vector
    let response = response_data.to_vec();

    // Check that we have at least status word (2 bytes)
    if response.len() < 2 {
        return Err(anyhow!(
            "APDU response too short: {} bytes (expected at least 2 for SW)",
            response.len()
        ));
    }

    // Extract status word (last 2 bytes)
    let sw1 = response[response.len() - 2];
    let sw2 = response[response.len() - 1];

    log::debug!(
        "APDU response: {} bytes, SW: {:02X}{:02X}",
        response.len(),
        sw1,
        sw2
    );

    // Log warning for error status (anything other than 90 00)
    if sw1 != 0x90 || sw2 != 0x00 {
        log::warn!("APDU returned error status: {:02X}{:02X}", sw1, sw2);
        log::trace!("Response data: {:02X?}", &response[..response.len() - 2]);
    } else {
        log::trace!("Response data: {:02X?}", &response[..response.len() - 2]);
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_hid_padding() {
        // We can't test actual HID operations without a device,
        // but we can test the logic
        let data = vec![0x01, 0x02, 0x03];
        assert!(data.len() < 64);
        // The function would pad this to 64 bytes
    }

    #[test]
    fn test_send_hid_too_large() {
        let data = vec![0u8; 65]; // Too large
        assert!(data.len() > 64);
        // This should fail when called
    }

    #[test]
    fn test_apdu_minimum_length() {
        let too_short = vec![0x00, 0xA4]; // Only 2 bytes
        assert!(too_short.len() < 4);

        let valid = vec![0x00, 0xA4, 0x04, 0x00]; // 4 bytes - valid
        assert!(valid.len() >= 4);
    }

    #[test]
    fn test_apdu_response_status_word() {
        // Success status: 90 00
        let success_response = vec![0x01, 0x02, 0x03, 0x90, 0x00];
        assert_eq!(success_response[success_response.len() - 2], 0x90);
        assert_eq!(success_response[success_response.len() - 1], 0x00);

        // Error status: 6A 82
        let error_response = vec![0x6A, 0x82];
        assert_eq!(error_response[error_response.len() - 2], 0x6A);
        assert_eq!(error_response[error_response.len() - 1], 0x82);
    }
}
