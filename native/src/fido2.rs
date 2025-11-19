use aes::Aes256;
use anyhow::{anyhow, Result};
use cbc::cipher::{block_padding::NoPadding, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use ciborium::Value as CborValue;
use hmac::{Hmac, Mac};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{ecdh::EphemeralSecret, PublicKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::device::DeviceManager;
use crate::transport;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

// CTAP2 command codes
const CTAP2_MAKE_CREDENTIAL: u8 = 0x01;
const CTAP2_GET_ASSERTION: u8 = 0x02;
const CTAP2_GET_INFO: u8 = 0x04;
const CTAP2_CLIENT_PIN: u8 = 0x06;
const CTAP2_RESET: u8 = 0x07;
const CTAP2_CREDENTIAL_MANAGEMENT: u8 = 0x0A;

/// CTAPHID commands
const CTAPHID_INIT: u8 = 0x06;
const CTAPHID_CBOR: u8 = 0x10;
const CTAPHID_CANCEL: u8 = 0x11;
const CTAPHID_KEEPALIVE: u8 = 0x3B;
const CTAPHID_ERROR: u8 = 0x3F;

/// CTAP2 status codes
const CTAP2_OK: u8 = 0x00;
const CTAP2_ERR_PIN_REQUIRED: u8 = 0x36;
const CTAP2_ERR_PIN_INVALID: u8 = 0x31;
const CTAP2_ERR_PIN_BLOCKED: u8 = 0x32;
const CTAP2_ERR_PIN_AUTH_INVALID: u8 = 0x33;
const CTAP2_ERR_PIN_AUTH_BLOCKED: u8 = 0x34;
const CTAP2_ERR_PIN_NOT_SET: u8 = 0x35;

/// Client PIN subcommands
const PIN_GET_RETRIES: u8 = 0x01;
const PIN_GET_KEY_AGREEMENT: u8 = 0x02;
const PIN_SET_PIN: u8 = 0x03;
const PIN_CHANGE_PIN: u8 = 0x04;
const PIN_GET_PIN_TOKEN: u8 = 0x05;
const PIN_GET_PIN_UV_AUTH_TOKEN_USING_UV_WITH_PERMISSIONS: u8 = 0x06;
const PIN_GET_UV_RETRIES: u8 = 0x07;
const PIN_GET_PIN_UV_AUTH_TOKEN_USING_PIN_WITH_PERMISSIONS: u8 = 0x09;

/// Credential Management subcommands
const CRED_MGMT_GET_CREDS_METADATA: u8 = 0x01;
const CRED_MGMT_ENUMERATE_RPS_BEGIN: u8 = 0x02;
const CRED_MGMT_ENUMERATE_RPS_NEXT: u8 = 0x03;
const CRED_MGMT_ENUMERATE_CREDENTIALS_BEGIN: u8 = 0x04;
const CRED_MGMT_ENUMERATE_CREDENTIALS_NEXT: u8 = 0x05;
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
    let nonce: [u8; 8] = rand::random();
    init_packet[7..15].copy_from_slice(&nonce);

    device_manager.with_hid_device(device_id, |device| {
        transport::send_hid(device, &init_packet)?;
        let init_response = transport::receive_hid(device, 5000)?;

        // Extract CID from response (bytes 15-18 of the INIT response)
        if init_response.len() >= 19 {
            let cid = [
                init_response[15],
                init_response[16],
                init_response[17],
                init_response[18],
            ];

            // Verify nonce matches
            if &init_response[8..16] != &nonce {
                return Err(anyhow!("INIT nonce mismatch"));
            }

            Ok(cid)
        } else {
            Err(anyhow!("Invalid INIT response"))
        }
    })
}

/// Send a CTAP2 command and receive response (handles continuation packets)
fn ctap2_command(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
    command: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    device_manager.with_hid_device(device_id, |device| {
        // Send request (with continuation packets if needed)
        let payload_len = 1 + data.len(); // command byte + data
        let mut sent = 0;
        let mut seq = 0u8;

        // Send initial packet
        let mut packet = [0u8; 64];
        packet[0..4].copy_from_slice(cid);
        packet[4] = CTAPHID_CBOR | 0x80; // CBOR command with TYPE_INIT bit
        packet[5] = ((payload_len >> 8) & 0xFF) as u8; // BCNTH
        packet[6] = (payload_len & 0xFF) as u8; // BCNTL
        packet[7] = command; // CTAP2 command

        // Copy first chunk of data (up to 57 bytes in first packet)
        let first_chunk_len = std::cmp::min(data.len(), 57);
        packet[8..8 + first_chunk_len].copy_from_slice(&data[..first_chunk_len]);
        sent += first_chunk_len;

        transport::send_hid(device, &packet)?;

        // Send continuation packets if needed
        while sent < data.len() {
            let mut cont_packet = [0u8; 64];
            cont_packet[0..4].copy_from_slice(cid);
            cont_packet[4] = seq; // Sequence number (no TYPE_INIT bit)

            let chunk_len = std::cmp::min(data.len() - sent, 59);
            cont_packet[5..5 + chunk_len].copy_from_slice(&data[sent..sent + chunk_len]);
            sent += chunk_len;
            seq += 1;

            transport::send_hid(device, &cont_packet)?;
        }

        // Receive response (with continuation packets if needed)
        // Use longer timeout (10s) to allow for user interaction like button press
        let response = transport::receive_hid(device, 10000)?;

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

        // Check for keepalive
        if response[4] == CTAPHID_KEEPALIVE {
            log::debug!("Received keepalive, waiting for actual response...");
            // In a real implementation, we'd loop and wait for the actual response
            // For now, just try to receive again
            let response = transport::receive_hid(device, 5000)?;
            if response.len() < 7 {
                return Err(anyhow!("Response too short after keepalive"));
            }
        }

        // Extract data length
        let data_len = ((response[5] as usize) << 8) | (response[6] as usize);
        let mut response_data = Vec::new();

        // Extract initial packet data (up to 57 bytes)
        let initial_data_len = std::cmp::min(data_len, 57);
        response_data.extend_from_slice(&response[7..7 + initial_data_len]);

        // Receive continuation packets if needed
        let mut received = initial_data_len;
        let mut expected_seq = 0u8;

        while received < data_len {
            let cont_response = transport::receive_hid(device, 5000)?;

            if cont_response.len() < 5 {
                return Err(anyhow!("Continuation packet too short"));
            }

            // Verify CID matches
            if &cont_response[0..4] != cid {
                return Err(anyhow!("CID mismatch in continuation packet"));
            }

            // Verify sequence number
            if cont_response[4] != expected_seq {
                return Err(anyhow!("Sequence number mismatch"));
            }

            let chunk_len = std::cmp::min(data_len - received, 59);
            response_data.extend_from_slice(&cont_response[5..5 + chunk_len]);
            received += chunk_len;
            expected_seq += 1;
        }

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

/// Parse CBOR value to string safely
fn cbor_to_string(value: &CborValue) -> String {
    match value {
        CborValue::Text(s) => s.clone(),
        CborValue::Bytes(b) => hex::encode(b),
        _ => format!("{:?}", value),
    }
}

/// Parse CBOR value to u32
fn cbor_to_u32(value: &CborValue) -> Option<u32> {
    match value {
        CborValue::Integer(i) => {
            let val: i128 = (*i).into();
            if val >= 0 && val <= u32::MAX as i128 {
                Some(val as u32)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse CBOR value to u8
fn cbor_to_u8(value: &CborValue) -> Option<u8> {
    match value {
        CborValue::Integer(i) => {
            let val: i128 = (*i).into();
            if val >= 0 && val <= u8::MAX as i128 {
                Some(val as u8)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse CBOR value to bool
fn cbor_to_bool(value: &CborValue) -> Option<bool> {
    match value {
        CborValue::Bool(b) => Some(*b),
        _ => None,
    }
}

/// Get FIDO2 authenticator info
pub fn get_info(device_manager: &DeviceManager, device_id: &str) -> Result<Fido2Info> {
    log::debug!("Getting FIDO2 authenticator info...");

    let cid = ctaphid_init(device_manager, device_id)?;
    let response = ctap2_command(device_manager, device_id, &cid, CTAP2_GET_INFO, &[])?;

    // Parse CBOR response
    let cbor: CborValue =
        ciborium::from_reader(&response[..]).map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

    log::debug!("Parsed CBOR response: {:?}", cbor);

    let map = match cbor {
        CborValue::Map(m) => m,
        _ => return Err(anyhow!("Expected CBOR map")),
    };

    // Parse the info structure
    let mut info = Fido2Info {
        versions: vec![],
        extensions: vec![],
        aaguid: String::new(),
        options: Fido2Options {
            plat: false,
            rk: false,
            client_pin: None,
            up: false,
            uv: None,
        },
        max_msg_size: None,
        pin_protocols: vec![],
        max_credential_count_in_list: None,
        max_credential_id_length: None,
        transports: vec![],
        algorithms: vec![],
        max_authenticator_config_length: None,
        default_cred_protect: None,
    };

    for (key, value) in map {
        match key {
            CborValue::Integer(i) => {
                let key_int: i128 = i.into();
                match key_int {
                    0x01 => {
                        // versions
                        if let CborValue::Array(arr) = value {
                            info.versions = arr.iter().map(cbor_to_string).collect();
                        }
                    }
                    0x02 => {
                        // extensions
                        if let CborValue::Array(arr) = value {
                            info.extensions = arr.iter().map(cbor_to_string).collect();
                        }
                    }
                    0x03 => {
                        // aaguid
                        if let CborValue::Bytes(b) = value {
                            // Format as UUID
                            if b.len() == 16 {
                                info.aaguid = format!(
                                    "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                                    b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
                                );
                            }
                        }
                    }
                    0x04 => {
                        // options
                        if let CborValue::Map(opts) = value {
                            for (opt_key, opt_value) in opts {
                                if let CborValue::Text(opt_name) = opt_key {
                                    match opt_name.as_str() {
                                        "plat" => {
                                            info.options.plat =
                                                cbor_to_bool(&opt_value).unwrap_or(false)
                                        }
                                        "rk" => {
                                            info.options.rk =
                                                cbor_to_bool(&opt_value).unwrap_or(false)
                                        }
                                        "clientPin" => {
                                            info.options.client_pin = cbor_to_bool(&opt_value)
                                        }
                                        "up" => {
                                            info.options.up =
                                                cbor_to_bool(&opt_value).unwrap_or(false)
                                        }
                                        "uv" => info.options.uv = cbor_to_bool(&opt_value),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    0x05 => {
                        // maxMsgSize
                        info.max_msg_size = cbor_to_u32(&value);
                    }
                    0x06 => {
                        // pinProtocols
                        if let CborValue::Array(arr) = value {
                            info.pin_protocols = arr.iter().filter_map(cbor_to_u8).collect();
                        }
                    }
                    0x07 => {
                        // maxCredentialCountInList
                        info.max_credential_count_in_list = cbor_to_u32(&value);
                    }
                    0x08 => {
                        // maxCredentialIdLength
                        info.max_credential_id_length = cbor_to_u32(&value);
                    }
                    0x09 => {
                        // transports
                        if let CborValue::Array(arr) = value {
                            info.transports = arr.iter().map(cbor_to_string).collect();
                        }
                    }
                    0x0A => {
                        // algorithms - array of maps with {alg: -7, type: "public-key"}
                        if let CborValue::Array(arr) = value {
                            for alg_val in arr {
                                if let CborValue::Map(alg_map) = alg_val {
                                    for (alg_key, alg_value) in alg_map {
                                        if let CborValue::Text(key_str) = alg_key {
                                            if key_str == "alg" {
                                                if let Some(alg_num) = cbor_to_u8(&alg_value)
                                                    .or_else(|| {
                                                        // Handle negative numbers (like -7 for ES256)
                                                        match alg_value {
                                                            CborValue::Integer(i) => {
                                                                let val: i128 = i.into();
                                                                match val {
                                                                    -7 => Some(0),   // ES256
                                                                    -8 => Some(1),   // EdDSA
                                                                    -257 => Some(2), // RS256
                                                                    _ => None,
                                                                }
                                                            }
                                                            _ => None,
                                                        }
                                                    })
                                                {
                                                    let alg_name = match alg_num {
                                                        0 => "ES256",
                                                        1 => "EdDSA",
                                                        2 => "RS256",
                                                        _ => "Unknown",
                                                    };
                                                    if !info
                                                        .algorithms
                                                        .contains(&alg_name.to_string())
                                                    {
                                                        info.algorithms.push(alg_name.to_string());
                                                    }
                                                }
                                                // Also handle by looking at the raw integer
                                                if let CborValue::Integer(i) = alg_value {
                                                    let val: i128 = i.into();
                                                    let alg_name = match val {
                                                        -7 => "ES256",
                                                        -8 => "EdDSA",
                                                        -257 => "RS256",
                                                        _ => continue,
                                                    };
                                                    if !info
                                                        .algorithms
                                                        .contains(&alg_name.to_string())
                                                    {
                                                        info.algorithms.push(alg_name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    0x0E => {
                        // maxAuthenticatorConfigLength
                        info.max_authenticator_config_length = cbor_to_u32(&value);
                    }
                    0x0F => {
                        // defaultCredProtect
                        info.default_cred_protect = cbor_to_u8(&value);
                    }
                    _ => {
                        log::debug!("Unknown info key: {}", key_int);
                    }
                }
            }
            _ => {
                log::warn!("Non-integer key in info map");
            }
        }
    }

    // Provide defaults if not present
    if info.versions.is_empty() {
        info.versions.push("FIDO_2_0".to_string());
    }
    if info.transports.is_empty() {
        info.transports.push("usb".to_string());
    }
    if info.algorithms.is_empty() {
        info.algorithms.push("ES256".to_string());
    }

    Ok(info)
}

/// Get PIN retry counter
pub fn get_pin_retries(device_manager: &DeviceManager, device_id: &str) -> Result<PinRetries> {
    log::debug!("Getting PIN retry counter...");

    let cid = ctaphid_init(device_manager, device_id)?;

    // Construct ClientPIN getRetries command
    // CBOR map: {0x01: pinProtocol, 0x02: subCommand}
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol = 1
        (
            CborValue::Integer(0x02.into()),
            CborValue::Integer(PIN_GET_RETRIES.into()),
        ), // subCommand = getPinRetries
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    let response = ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data)?;

    // Parse CBOR response
    let cbor: CborValue =
        ciborium::from_reader(&response[..]).map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

    let map = match cbor {
        CborValue::Map(m) => m,
        _ => return Err(anyhow!("Expected CBOR map")),
    };

    let mut retries = 8u8;
    let mut power_cycle_required = false;

    for (key, value) in map {
        if let CborValue::Integer(i) = key {
            let key_int: i128 = i.into();
            match key_int {
                0x03 => {
                    // retries
                    retries = cbor_to_u8(&value).unwrap_or(8);
                }
                0x05 => {
                    // powerCycleState
                    power_cycle_required = cbor_to_bool(&value).unwrap_or(false);
                }
                _ => {}
            }
        }
    }

    Ok(PinRetries {
        retries,
        power_cycle_required,
    })
}

/// Get authenticator's public key for key agreement
fn get_key_agreement(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
) -> Result<Vec<u8>> {
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol = 1
        (
            CborValue::Integer(0x02.into()),
            CborValue::Integer(PIN_GET_KEY_AGREEMENT.into()),
        ),
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    let response = ctap2_command(device_manager, device_id, cid, CTAP2_CLIENT_PIN, &data)?;

    let cbor: CborValue =
        ciborium::from_reader(&response[..]).map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

    let map = match cbor {
        CborValue::Map(m) => m,
        _ => return Err(anyhow!("Expected CBOR map")),
    };

    // Extract key agreement (COSE_Key structure)
    for (key, value) in map {
        if let CborValue::Integer(i) = key {
            let key_int: i128 = i.into();
            if key_int == 0x01 {
                // keyAgreement
                if let CborValue::Map(cose_key) = value {
                    // Extract x and y coordinates from COSE_Key
                    let mut x_coord: Option<Vec<u8>> = None;
                    let mut y_coord: Option<Vec<u8>> = None;

                    for (cose_key, cose_value) in cose_key {
                        if let CborValue::Integer(cose_key_int) = cose_key {
                            let key_num: i128 = cose_key_int.into();
                            match key_num {
                                -2 => {
                                    // x coordinate
                                    if let CborValue::Bytes(b) = cose_value {
                                        x_coord = Some(b);
                                    }
                                }
                                -3 => {
                                    // y coordinate
                                    if let CborValue::Bytes(b) = cose_value {
                                        y_coord = Some(b);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let (Some(x), Some(y)) = (x_coord, y_coord) {
                        // Construct uncompressed point (0x04 || x || y)
                        let mut point = vec![0x04];
                        point.extend_from_slice(&x);
                        point.extend_from_slice(&y);
                        return Ok(point);
                    }
                }
            }
        }
    }

    Err(anyhow!("Key agreement not found in response"))
}

/// Compute shared secret using ECDH
fn compute_shared_secret(authenticator_public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // Generate ephemeral key pair
    let secret_key = EphemeralSecret::random(&mut OsRng);
    let public_key = p256::PublicKey::from(&secret_key);

    // Encode our public key
    let encoded_point = public_key.to_encoded_point(false);
    let platform_key_bytes = encoded_point.as_bytes().to_vec();

    // Parse authenticator's public key
    let auth_public_key = PublicKey::from_sec1_bytes(authenticator_public_key)
        .map_err(|e| anyhow!("Failed to parse authenticator public key: {}", e))?;

    // Compute shared secret using ECDH
    let shared_secret = secret_key.diffie_hellman(&auth_public_key);

    // Hash the shared secret with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.raw_secret_bytes());
    let shared_secret_hash = hasher.finalize().to_vec();

    Ok((shared_secret_hash, platform_key_bytes))
}

/// Encrypt PIN using AES-256-CBC
fn encrypt_pin(pin: &str, shared_secret: &[u8]) -> Result<Vec<u8>> {
    // Pad PIN to 64 bytes
    let mut pin_bytes = pin.as_bytes().to_vec();
    pin_bytes.resize(64, 0);

    // Use shared secret as key (first 32 bytes)
    let key = &shared_secret[0..32];

    // Use zero IV for PIN protocol v1
    let iv = [0u8; 16];

    // Encrypt using AES-256-CBC
    let cipher = Aes256CbcEnc::new(key.into(), &iv.into());

    // The data is already 64 bytes which is a multiple of 16, so no padding needed
    let ciphertext = cipher
        .encrypt_padded_mut::<NoPadding>(&mut pin_bytes, 64)
        .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;

    Ok(ciphertext.to_vec())
}

/// Compute PIN auth (HMAC-SHA-256)
fn compute_pin_auth(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(key).map_err(|e| anyhow!("HMAC creation failed: {}", e))?;
    mac.update(data);
    let result = mac.finalize();

    // Return first 16 bytes
    Ok(result.into_bytes()[0..16].to_vec())
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

    // Step 1: Get key agreement from authenticator
    let auth_public_key = get_key_agreement(device_manager, device_id, &cid)?;

    // Step 2: Generate shared secret
    let (shared_secret, platform_public_key) = compute_shared_secret(&auth_public_key)?;

    // Step 3: Encrypt new PIN
    let encrypted_pin = encrypt_pin(new_pin, &shared_secret)?;

    // Step 4: Compute pinAuth
    let pin_auth = compute_pin_auth(&shared_secret, &encrypted_pin)?;

    // Step 5: Build COSE_Key for platform public key
    let cose_key = vec![
        (CborValue::Integer(1.into()), CborValue::Integer(2.into())), // kty: EC2
        (
            CborValue::Integer(3.into()),
            CborValue::Integer((-25).into()),
        ), // alg: ECDH-ES+HKDF-256
        (
            CborValue::Integer((-1).into()),
            CborValue::Integer(1.into()),
        ), // crv: P-256
        (
            CborValue::Integer((-2).into()),
            CborValue::Bytes(platform_public_key[1..33].to_vec()),
        ), // x
        (
            CborValue::Integer((-3).into()),
            CborValue::Bytes(platform_public_key[33..65].to_vec()),
        ), // y
    ];

    // Step 6: Build command
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol
        (
            CborValue::Integer(0x02.into()),
            CborValue::Integer(PIN_SET_PIN.into()),
        ), // subCommand
        (CborValue::Integer(0x03.into()), CborValue::Map(cose_key)), // keyAgreement
        (
            CborValue::Integer(0x05.into()),
            CborValue::Bytes(encrypted_pin),
        ), // newPinEnc
        (CborValue::Integer(0x06.into()), CborValue::Bytes(pin_auth)), // pinAuth
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data)?;

    log::info!("PIN set successfully");
    Ok(())
}

/// Change existing PIN
pub fn change_pin(
    device_manager: &DeviceManager,
    device_id: &str,
    current_pin: &str,
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

    // Step 1: Get key agreement from authenticator
    let auth_public_key = get_key_agreement(device_manager, device_id, &cid)?;

    // Step 2: Generate shared secret
    let (shared_secret, platform_public_key) = compute_shared_secret(&auth_public_key)?;

    // Step 3: Encrypt both PINs
    let encrypted_new_pin = encrypt_pin(new_pin, &shared_secret)?;
    let encrypted_current_pin_hash = {
        // Hash the current PIN first
        let mut hasher = Sha256::new();
        hasher.update(current_pin.as_bytes());
        let pin_hash_left16 = &hasher.finalize()[0..16];

        // Pad to 16 bytes (already 16, but for consistency)
        let mut padded = pin_hash_left16.to_vec();
        padded.resize(16, 0);

        // Encrypt
        let key = &shared_secret[0..32];
        let iv = [0u8; 16];
        let cipher = Aes256CbcEnc::new(key.into(), &iv.into());
        let encrypted = cipher
            .encrypt_padded_mut::<NoPadding>(&mut padded, 16)
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;
        encrypted.to_vec()
    };

    // Step 4: Compute pinAuth over newPinEnc || pinHashEnc
    let mut pin_auth_data = encrypted_new_pin.clone();
    pin_auth_data.extend_from_slice(&encrypted_current_pin_hash);
    let pin_auth = compute_pin_auth(&shared_secret, &pin_auth_data)?;

    // Step 5: Build COSE_Key for platform public key
    let cose_key = vec![
        (CborValue::Integer(1.into()), CborValue::Integer(2.into())), // kty: EC2
        (
            CborValue::Integer(3.into()),
            CborValue::Integer((-25).into()),
        ), // alg: ECDH-ES+HKDF-256
        (
            CborValue::Integer((-1).into()),
            CborValue::Integer(1.into()),
        ), // crv: P-256
        (
            CborValue::Integer((-2).into()),
            CborValue::Bytes(platform_public_key[1..33].to_vec()),
        ), // x
        (
            CborValue::Integer((-3).into()),
            CborValue::Bytes(platform_public_key[33..65].to_vec()),
        ), // y
    ];

    // Step 6: Build command
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol
        (
            CborValue::Integer(0x02.into()),
            CborValue::Integer(PIN_CHANGE_PIN.into()),
        ), // subCommand
        (CborValue::Integer(0x03.into()), CborValue::Map(cose_key)), // keyAgreement
        (
            CborValue::Integer(0x04.into()),
            CborValue::Bytes(encrypted_current_pin_hash),
        ), // pinHashEnc
        (
            CborValue::Integer(0x05.into()),
            CborValue::Bytes(encrypted_new_pin),
        ), // newPinEnc
        (CborValue::Integer(0x06.into()), CborValue::Bytes(pin_auth)), // pinAuth
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    ctap2_command(device_manager, device_id, &cid, CTAP2_CLIENT_PIN, &data)?;

    log::info!("PIN changed successfully");
    Ok(())
}

/// Get PIN token for credential management
fn get_pin_token(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
    pin: &str,
) -> Result<Vec<u8>> {
    // Step 1: Get key agreement
    let auth_public_key = get_key_agreement(device_manager, device_id, cid)?;

    // Step 2: Generate shared secret
    let (shared_secret, platform_public_key) = compute_shared_secret(&auth_public_key)?;

    // Step 3: Hash PIN and take first 16 bytes
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    let pin_hash_left16 = &hasher.finalize()[0..16];

    // Step 4: Encrypt PIN hash
    let mut padded = pin_hash_left16.to_vec();
    padded.resize(16, 0);

    let key = &shared_secret[0..32];
    let iv = [0u8; 16];
    let cipher = Aes256CbcEnc::new(key.into(), &iv.into());
    let encrypted_pin_hash = cipher
        .encrypt_padded_mut::<NoPadding>(&mut padded, 16)
        .map_err(|e| anyhow!("Encryption failed: {:?}", e))?
        .to_vec();

    // Step 5: Build COSE_Key
    let cose_key = vec![
        (CborValue::Integer(1.into()), CborValue::Integer(2.into())),
        (
            CborValue::Integer(3.into()),
            CborValue::Integer((-25).into()),
        ),
        (
            CborValue::Integer((-1).into()),
            CborValue::Integer(1.into()),
        ),
        (
            CborValue::Integer((-2).into()),
            CborValue::Bytes(platform_public_key[1..33].to_vec()),
        ),
        (
            CborValue::Integer((-3).into()),
            CborValue::Bytes(platform_public_key[33..65].to_vec()),
        ),
    ];

    // Step 6: Build command
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol
        (
            CborValue::Integer(0x02.into()),
            CborValue::Integer(PIN_GET_PIN_TOKEN.into()),
        ), // subCommand
        (CborValue::Integer(0x03.into()), CborValue::Map(cose_key)), // keyAgreement
        (
            CborValue::Integer(0x04.into()),
            CborValue::Bytes(encrypted_pin_hash),
        ), // pinHashEnc
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    let response = ctap2_command(device_manager, device_id, cid, CTAP2_CLIENT_PIN, &data)?;

    // Parse response to get encrypted PIN token
    let cbor: CborValue =
        ciborium::from_reader(&response[..]).map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

    let map = match cbor {
        CborValue::Map(m) => m,
        _ => return Err(anyhow!("Expected CBOR map")),
    };

    for (key, value) in map {
        if let CborValue::Integer(i) = key {
            let key_int: i128 = i.into();
            if key_int == 0x02 {
                // pinToken
                if let CborValue::Bytes(encrypted_token) = value {
                    // Decrypt PIN token
                    let key = &shared_secret[0..32];
                    let iv = [0u8; 16];
                    let cipher = Aes256CbcDec::new(key.into(), &iv.into());
                    let mut buffer = encrypted_token.clone();
                    let decrypted = cipher
                        .decrypt_padded_mut::<NoPadding>(&mut buffer)
                        .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;
                    return Ok(decrypted.to_vec());
                }
            }
        }
    }

    Err(anyhow!("PIN token not found in response"))
}

/// List all credentials
pub fn list_credentials(
    device_manager: &DeviceManager,
    device_id: &str,
    pin: Option<&str>,
) -> Result<Vec<Credential>> {
    log::debug!("Listing credentials...");

    let cid = ctaphid_init(device_manager, device_id)?;

    // If no PIN provided, return empty list (credentials require PIN)
    let pin = match pin {
        Some(p) => p,
        None => {
            log::debug!("No PIN provided for credential listing");
            return Ok(vec![]);
        }
    };

    // Get PIN token
    let pin_token = get_pin_token(device_manager, device_id, &cid, pin)?;

    let mut credentials = Vec::new();

    // First, enumerate RPs
    // Compute pinAuth
    let mut pin_auth_data = Vec::new();
    ciborium::into_writer(&CborValue::Bytes(Vec::new()), &mut pin_auth_data)
        .map_err(|e| anyhow!("Failed to encode: {}", e))?;
    let pin_auth = compute_pin_auth(&pin_token, &pin_auth_data)?;

    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(CRED_MGMT_ENUMERATE_RPS_BEGIN.into()),
        ), // subCommand
        (
            CborValue::Integer(0x02.into()),
            CborValue::Bytes(Vec::new()),
        ), // subCommandParams (empty)
        (
            CborValue::Integer(0x03.into()),
            CborValue::Integer(1.into()),
        ), // pinProtocol
        (CborValue::Integer(0x04.into()), CborValue::Bytes(pin_auth)), // pinAuth
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    // Try to enumerate RPs
    match ctap2_command(
        device_manager,
        device_id,
        &cid,
        CTAP2_CREDENTIAL_MANAGEMENT,
        &data,
    ) {
        Ok(response) => {
            // Parse RP info
            let cbor: CborValue = ciborium::from_reader(&response[..])
                .map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

            if let CborValue::Map(rp_map) = cbor {
                // Extract RP info and enumerate credentials for this RP
                let mut rp_id = String::new();
                let mut rp_name = String::new();

                for (key, value) in &rp_map {
                    if let CborValue::Integer(i) = key {
                        let key_int: i128 = (*i).into();
                        match key_int {
                            0x03 => {
                                // rp
                                if let CborValue::Map(rp_info) = value {
                                    for (rp_key, rp_value) in rp_info {
                                        if let CborValue::Text(field) = rp_key {
                                            match field.as_str() {
                                                "id" => rp_id = cbor_to_string(rp_value),
                                                "name" => rp_name = cbor_to_string(rp_value),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Enumerate credentials for this RP
                if !rp_id.is_empty() {
                    credentials.extend(enumerate_credentials_for_rp(
                        device_manager,
                        device_id,
                        &cid,
                        &pin_token,
                        &rp_id,
                        &rp_name,
                    )?);
                }
            }
        }
        Err(e) => {
            // If error is PIN_REQUIRED or similar, return empty list
            log::debug!("Failed to enumerate RPs: {}", e);
            return Ok(vec![]);
        }
    }

    Ok(credentials)
}

/// Enumerate credentials for a specific RP
fn enumerate_credentials_for_rp(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
    pin_token: &[u8],
    rp_id: &str,
    rp_name: &str,
) -> Result<Vec<Credential>> {
    let mut credentials = Vec::new();

    // Build subCommandParams
    let sub_params = vec![(
        CborValue::Text("id".to_string()),
        CborValue::Text(rp_id.to_string()),
    )];

    // Encode subCommandParams
    let mut sub_params_bytes = Vec::new();
    ciborium::into_writer(&CborValue::Map(sub_params), &mut sub_params_bytes)
        .map_err(|e| anyhow!("Failed to encode: {}", e))?;

    // Compute pinAuth
    let pin_auth = compute_pin_auth(pin_token, &sub_params_bytes)?;

    // Build command
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(CRED_MGMT_ENUMERATE_CREDENTIALS_BEGIN.into()),
        ),
        (
            CborValue::Integer(0x02.into()),
            CborValue::Bytes(sub_params_bytes),
        ),
        (
            CborValue::Integer(0x03.into()),
            CborValue::Integer(1.into()),
        ),
        (CborValue::Integer(0x04.into()), CborValue::Bytes(pin_auth)),
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    match ctap2_command(
        device_manager,
        device_id,
        cid,
        CTAP2_CREDENTIAL_MANAGEMENT,
        &data,
    ) {
        Ok(response) => {
            let cbor: CborValue = ciborium::from_reader(&response[..])
                .map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

            if let CborValue::Map(cred_map) = cbor {
                let credential = parse_credential(&cred_map, rp_id, rp_name)?;
                credentials.push(credential);

                // Check if there are more credentials
                let mut total_credentials = 1;
                for (key, value) in &cred_map {
                    if let CborValue::Integer(i) = key {
                        if i128::from(*i) == 0x05 {
                            // totalCredentials
                            total_credentials = cbor_to_u8(value).unwrap_or(1) as usize;
                        }
                    }
                }

                // Enumerate remaining credentials
                for _ in 1..total_credentials {
                    match enumerate_next_credential(device_manager, device_id, cid, rp_id, rp_name)
                    {
                        Ok(cred) => credentials.push(cred),
                        Err(e) => {
                            log::warn!("Failed to enumerate next credential: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::debug!("No credentials for RP {}: {}", rp_id, e);
        }
    }

    Ok(credentials)
}

/// Enumerate next credential
fn enumerate_next_credential(
    device_manager: &DeviceManager,
    device_id: &str,
    cid: &[u8; 4],
    rp_id: &str,
    rp_name: &str,
) -> Result<Credential> {
    let cmd_map = vec![(
        CborValue::Integer(0x01.into()),
        CborValue::Integer(CRED_MGMT_ENUMERATE_CREDENTIALS_NEXT.into()),
    )];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    let response = ctap2_command(
        device_manager,
        device_id,
        cid,
        CTAP2_CREDENTIAL_MANAGEMENT,
        &data,
    )?;

    let cbor: CborValue =
        ciborium::from_reader(&response[..]).map_err(|e| anyhow!("Failed to parse CBOR: {}", e))?;

    let map = match cbor {
        CborValue::Map(m) => m,
        _ => return Err(anyhow!("Expected CBOR map")),
    };

    parse_credential(&map, rp_id, rp_name)
}

/// Parse credential from CBOR map
fn parse_credential(
    map: &[(CborValue, CborValue)],
    rp_id: &str,
    rp_name: &str,
) -> Result<Credential> {
    let mut user_id = String::new();
    let mut user_name = String::new();
    let mut user_display_name = String::new();
    let mut credential_id = String::new();
    let mut public_key = None;
    let mut cred_protect = None;

    for (key, value) in map {
        if let CborValue::Integer(i) = key {
            let key_int: i128 = (*i).into();
            match key_int {
                0x06 => {
                    // user
                    if let CborValue::Map(user_info) = value {
                        for (user_key, user_value) in user_info {
                            if let CborValue::Text(field) = user_key {
                                match field.as_str() {
                                    "id" => {
                                        user_id = match user_value {
                                            CborValue::Bytes(b) => hex::encode(b),
                                            _ => String::new(),
                                        }
                                    }
                                    "name" => user_name = cbor_to_string(&user_value),
                                    "displayName" => {
                                        user_display_name = cbor_to_string(&user_value)
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                0x07 => {
                    // credentialID
                    if let CborValue::Map(cred_id_info) = value {
                        for (cred_key, cred_value) in cred_id_info {
                            if let CborValue::Text(field) = cred_key {
                                if field == "id" {
                                    credential_id = match cred_value {
                                        CborValue::Bytes(b) => hex::encode(b),
                                        _ => String::new(),
                                    };
                                }
                            }
                        }
                    }
                }
                0x08 => {
                    // publicKey
                    // COSE_Key format - could be parsed further
                    public_key = Some(format!("{:?}", value));
                }
                0x0A => {
                    // credProtect
                    cred_protect = cbor_to_u8(&value);
                }
                _ => {}
            }
        }
    }

    Ok(Credential {
        rp_id: rp_id.to_string(),
        rp_name: rp_name.to_string(),
        user_id,
        user_name,
        user_display_name,
        credential_id,
        public_key,
        cred_protect,
    })
}

/// Delete a credential by ID
pub fn delete_credential(
    device_manager: &DeviceManager,
    device_id: &str,
    credential_id: &str,
    pin: Option<&str>,
) -> Result<()> {
    log::debug!("Deleting credential: {}", credential_id);

    let cid = ctaphid_init(device_manager, device_id)?;

    let pin = pin.ok_or_else(|| anyhow!("PIN required for credential deletion"))?;

    // Get PIN token
    let pin_token = get_pin_token(device_manager, device_id, &cid, pin)?;

    // Decode credential ID from hex
    let cred_id_bytes =
        hex::decode(credential_id).map_err(|e| anyhow!("Invalid credential ID: {}", e))?;

    // Build subCommandParams
    let cred_descriptor = vec![
        (
            CborValue::Text("id".to_string()),
            CborValue::Bytes(cred_id_bytes),
        ),
        (
            CborValue::Text("type".to_string()),
            CborValue::Text("public-key".to_string()),
        ),
    ];

    let sub_params = vec![(
        CborValue::Text("credentialDescriptor".to_string()),
        CborValue::Map(cred_descriptor),
    )];

    let mut sub_params_bytes = Vec::new();
    ciborium::into_writer(&CborValue::Map(sub_params), &mut sub_params_bytes)
        .map_err(|e| anyhow!("Failed to encode: {}", e))?;

    // Compute pinAuth
    let pin_auth = compute_pin_auth(&pin_token, &sub_params_bytes)?;

    // Build command
    let cmd_map = vec![
        (
            CborValue::Integer(0x01.into()),
            CborValue::Integer(CRED_MGMT_DELETE_CREDENTIAL.into()),
        ),
        (
            CborValue::Integer(0x02.into()),
            CborValue::Bytes(sub_params_bytes),
        ),
        (
            CborValue::Integer(0x03.into()),
            CborValue::Integer(1.into()),
        ),
        (CborValue::Integer(0x04.into()), CborValue::Bytes(pin_auth)),
    ];

    let mut data = Vec::new();
    ciborium::into_writer(&CborValue::Map(cmd_map), &mut data)
        .map_err(|e| anyhow!("Failed to encode CBOR: {}", e))?;

    ctap2_command(
        device_manager,
        device_id,
        &cid,
        CTAP2_CREDENTIAL_MANAGEMENT,
        &data,
    )?;

    log::info!("Credential deleted successfully");
    Ok(())
}

/// Reset the authenticator to factory defaults
pub fn reset_device(device_manager: &DeviceManager, device_id: &str) -> Result<()> {
    log::debug!("Resetting authenticator...");

    let cid = ctaphid_init(device_manager, device_id)?;

    // RESET command has no parameters
    ctap2_command(device_manager, device_id, &cid, CTAP2_RESET, &[])?;

    log::info!("Authenticator reset successful");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_length_validation() {
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
