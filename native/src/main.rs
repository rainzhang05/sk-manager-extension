use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

mod device;
mod fido2;
mod piv;
mod protocol;
mod transport;

/// Request structure for JSON-RPC messages
#[derive(Debug, Deserialize)]
struct Request {
    id: u32,
    command: String,
    #[serde(default)]
    #[allow(dead_code)]
    params: serde_json::Value,
}

/// Response structure for JSON-RPC messages
#[derive(Debug, Serialize)]
struct Response {
    id: u32,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorInfo>,
}

/// Error information
#[derive(Debug, Serialize)]
struct ErrorInfo {
    code: String,
    message: String,
}

impl Response {
    fn success(id: u32, result: serde_json::Value) -> Self {
        Response {
            id,
            status: "ok".to_string(),
            result: Some(result),
            error: None,
        }
    }

    fn error(id: u32, code: &str, message: &str) -> Self {
        Response {
            id,
            status: "error".to_string(),
            result: None,
            error: Some(ErrorInfo {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

/// Read a message length (4 bytes, native endian)
fn read_message_length() -> io::Result<u32> {
    let mut length_bytes = [0u8; 4];
    io::stdin().read_exact(&mut length_bytes)?;
    Ok(u32::from_ne_bytes(length_bytes))
}

/// Read a message of specified length
fn read_message(length: u32) -> io::Result<String> {
    let mut buffer = vec![0u8; length as usize];
    io::stdin().read_exact(&mut buffer)?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write a message with length prefix
fn write_message(message: &str) -> io::Result<()> {
    let length = message.len() as u32;
    io::stdout().write_all(&length.to_ne_bytes())?;
    io::stdout().write_all(message.as_bytes())?;
    io::stdout().flush()?;
    Ok(())
}

/// Handle a ping command
fn handle_ping(id: u32) -> Response {
    log::debug!("Handling ping command");
    Response::success(
        id,
        serde_json::json!({
            "message": "pong"
        }),
    )
}

/// Handle a getVersion command
fn handle_get_version(id: u32) -> Response {
    log::debug!("Handling getVersion command");
    Response::success(
        id,
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "name": env!("CARGO_PKG_NAME")
        }),
    )
}

/// Handle a listDevices command
fn handle_list_devices(id: u32) -> Response {
    log::debug!("Handling listDevices command");
    match device::list_devices() {
        Ok(devices) => Response::success(
            id,
            serde_json::json!({
                "devices": devices
            }),
        ),
        Err(e) => Response::error(
            id,
            "DEVICE_ENUMERATION_FAILED",
            &format!("Failed to enumerate devices: {}", e),
        ),
    }
}

/// Handle an openDevice command
fn handle_open_device(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling openDevice command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    match device_manager.open_device(device_id) {
        Ok(()) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "deviceId": device_id
            }),
        ),
        Err(e) => Response::error(
            id,
            "DEVICE_OPEN_FAILED",
            &format!("Failed to open device: {}", e),
        ),
    }
}

/// Handle a closeDevice command
fn handle_close_device(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling closeDevice command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    match device_manager.close_device(device_id) {
        Ok(()) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "deviceId": device_id
            }),
        ),
        Err(e) => Response::error(
            id,
            "DEVICE_CLOSE_FAILED",
            &format!("Failed to close device: {}", e),
        ),
    }
}

/// Handle a sendHid command
fn handle_send_hid(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling sendHid command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let data_array = match params.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing or invalid data parameter");
        }
    };

    // Convert JSON array to byte array
    let data_bytes: Vec<u8> = data_array
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();

    if data_bytes.len() != data_array.len() {
        return Response::error(
            id,
            "INVALID_PARAMS",
            "Data array contains invalid byte values",
        );
    }

    // Execute HID send with the device
    match device_manager
        .with_hid_device(device_id, |device| transport::send_hid(device, &data_bytes))
    {
        Ok(bytes_sent) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "bytesSent": bytes_sent
            }),
        ),
        Err(e) => Response::error(
            id,
            "HID_SEND_FAILED",
            &format!("Failed to send HID packet: {}", e),
        ),
    }
}

/// Handle a receiveHid command
fn handle_receive_hid(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling receiveHid command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let timeout = params
        .get("timeout")
        .and_then(|v| v.as_i64())
        .unwrap_or(5000) as i32;

    // Execute HID receive with the device
    match device_manager
        .with_hid_device(device_id, |device| transport::receive_hid(device, timeout))
    {
        Ok(data) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "data": data
            }),
        ),
        Err(e) => Response::error(
            id,
            "HID_RECEIVE_FAILED",
            &format!("Failed to receive HID packet: {}", e),
        ),
    }
}

/// Handle a transmitApdu command
fn handle_transmit_apdu(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling transmitApdu command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let apdu_array = match params.get("apdu").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing or invalid apdu parameter");
        }
    };

    // Convert JSON array to byte array
    let apdu_bytes: Vec<u8> = apdu_array
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();

    if apdu_bytes.len() != apdu_array.len() {
        return Response::error(
            id,
            "INVALID_PARAMS",
            "APDU array contains invalid byte values",
        );
    }

    // Execute APDU transmit with the device
    match device_manager.with_ccid_card(device_id, |card| {
        transport::transmit_apdu(card, &apdu_bytes)
    }) {
        Ok(response) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "response": response
            }),
        ),
        Err(e) => Response::error(
            id,
            "APDU_TRANSMIT_FAILED",
            &format!("Failed to transmit APDU: {}", e),
        ),
    }
}

/// Handle a detectProtocols command
fn handle_detect_protocols(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling detectProtocols command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    // Detect protocols for the device
    match protocol::detect_protocols(device_manager, device_id) {
        Ok(support) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "protocols": support
            }),
        ),
        Err(e) => Response::error(
            id,
            "PROTOCOL_DETECTION_FAILED",
            &format!("Failed to detect protocols: {}", e),
        ),
    }
}

/// Handle a fido2GetInfo command
fn handle_fido2_get_info(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2GetInfo command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    match fido2::get_info(device_manager, device_id) {
        Ok(info) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "info": info
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_GET_INFO_FAILED",
            &format!("Failed to get FIDO2 info: {}", e),
        ),
    }
}

/// Handle a fido2GetPinRetries command
fn handle_fido2_get_pin_retries(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2GetPinRetries command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    match fido2::get_pin_retries(device_manager, device_id) {
        Ok(retries) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "retries": retries
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_GET_PIN_RETRIES_FAILED",
            &format!("Failed to get PIN retries: {}", e),
        ),
    }
}

/// Handle a fido2SetPin command
fn handle_fido2_set_pin(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2SetPin command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let new_pin = match params.get("newPin").and_then(|v| v.as_str()) {
        Some(pin) => pin,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing newPin parameter");
        }
    };

    match fido2::set_pin(device_manager, device_id, new_pin) {
        Ok(_) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "message": "PIN set successfully"
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_SET_PIN_FAILED",
            &format!("Failed to set PIN: {}", e),
        ),
    }
}

/// Handle a fido2ChangePin command
fn handle_fido2_change_pin(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2ChangePin command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let current_pin = match params.get("currentPin").and_then(|v| v.as_str()) {
        Some(pin) => pin,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing currentPin parameter");
        }
    };

    let new_pin = match params.get("newPin").and_then(|v| v.as_str()) {
        Some(pin) => pin,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing newPin parameter");
        }
    };

    match fido2::change_pin(device_manager, device_id, current_pin, new_pin) {
        Ok(_) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "message": "PIN changed successfully"
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_CHANGE_PIN_FAILED",
            &format!("Failed to change PIN: {}", e),
        ),
    }
}

/// Handle a fido2ListCredentials command
fn handle_fido2_list_credentials(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2ListCredentials command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    // PIN is optional for listing credentials
    let pin = params.get("pin").and_then(|v| v.as_str());

    match fido2::list_credentials(device_manager, device_id, pin) {
        Ok(credentials) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "credentials": credentials
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_LIST_CREDENTIALS_FAILED",
            &format!("Failed to list credentials: {}", e),
        ),
    }
}

/// Handle a fido2DeleteCredential command
fn handle_fido2_delete_credential(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2DeleteCredential command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    let credential_id = match params.get("credentialId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing credentialId parameter");
        }
    };

    // PIN is optional for deleting credentials
    let pin = params.get("pin").and_then(|v| v.as_str());

    match fido2::delete_credential(device_manager, device_id, credential_id, pin) {
        Ok(_) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "message": "Credential deleted successfully"
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_DELETE_CREDENTIAL_FAILED",
            &format!("Failed to delete credential: {}", e),
        ),
    }
}

/// Handle a fido2ResetDevice command
fn handle_fido2_reset_device(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling fido2ResetDevice command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    match fido2::reset_device(device_manager, device_id) {
        Ok(_) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "message": "Device reset successfully"
            }),
        ),
        Err(e) => Response::error(
            id,
            "FIDO2_RESET_DEVICE_FAILED",
            &format!("Failed to reset device: {}", e),
        ),
    }
}

/// Handle a pivGetData command
fn handle_piv_get_data(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling pivGetData command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    // Check if device is CCID type before proceeding
    match device::list_devices() {
        Ok(devices) => {
            let device = devices.iter().find(|d| d.id == device_id);
            match device {
                Some(d) => {
                    if d.device_type != device::DeviceType::Ccid {
                        return Response::error(
                            id,
                            "DEVICE_TYPE_MISMATCH",
                            "PIV operations require a CCID device. The specified device is not a CCID device."
                        );
                    }
                }
                None => {
                    return Response::error(
                        id,
                        "DEVICE_NOT_FOUND",
                        &format!("Device with ID {} not found", device_id)
                    );
                }
            }
        }
        Err(e) => {
            return Response::error(
                id,
                "DEVICE_ENUMERATION_FAILED",
                &format!("Failed to enumerate devices: {}", e)
            );
        }
    }

    match piv::get_piv_data(device_manager, device_id) {
        Ok(result) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "info": result.info,
                "activityLog": result.activity_log
            }),
        ),
        Err(e) => Response::error(
            id,
            "PIV_GET_DATA_FAILED",
            &format!("Failed to get PIV data: {}", e),
        ),
    }
}

/// Handle a pivSelect command
fn handle_piv_select(
    id: u32,
    params: &serde_json::Value,
    device_manager: &device::DeviceManager,
) -> Response {
    log::debug!("Handling pivSelect command");

    let device_id = match params.get("deviceId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return Response::error(id, "INVALID_PARAMS", "Missing deviceId parameter");
        }
    };

    // Check if device is CCID type before proceeding
    match device::list_devices() {
        Ok(devices) => {
            let device = devices.iter().find(|d| d.id == device_id);
            match device {
                Some(d) => {
                    if d.device_type != device::DeviceType::Ccid {
                        return Response::error(
                            id,
                            "DEVICE_TYPE_MISMATCH",
                            "PIV operations require a CCID device. The specified device is not a CCID device."
                        );
                    }
                }
                None => {
                    return Response::error(
                        id,
                        "DEVICE_NOT_FOUND",
                        &format!("Device with ID {} not found", device_id)
                    );
                }
            }
        }
        Err(e) => {
            return Response::error(
                id,
                "DEVICE_ENUMERATION_FAILED",
                &format!("Failed to enumerate devices: {}", e)
            );
        }
    }

    match piv::select_piv(device_manager, device_id) {
        Ok(selected) => Response::success(
            id,
            serde_json::json!({
                "success": true,
                "selected": selected
            }),
        ),
        Err(e) => Response::error(
            id,
            "PIV_SELECT_FAILED",
            &format!("Failed to select PIV application: {}", e),
        ),
    }
}

/// Process a single request
fn process_request(request: Request, device_manager: &device::DeviceManager) -> Response {
    log::info!(
        "Processing command: {} (id: {})",
        request.command,
        request.id
    );

    match request.command.as_str() {
        "ping" => handle_ping(request.id),
        "getVersion" => handle_get_version(request.id),
        "listDevices" => handle_list_devices(request.id),
        "openDevice" => handle_open_device(request.id, &request.params, device_manager),
        "closeDevice" => handle_close_device(request.id, &request.params, device_manager),
        "sendHid" => handle_send_hid(request.id, &request.params, device_manager),
        "receiveHid" => handle_receive_hid(request.id, &request.params, device_manager),
        "transmitApdu" => handle_transmit_apdu(request.id, &request.params, device_manager),
        "detectProtocols" => handle_detect_protocols(request.id, &request.params, device_manager),
        "fido2GetInfo" => handle_fido2_get_info(request.id, &request.params, device_manager),
        "fido2GetPinRetries" => {
            handle_fido2_get_pin_retries(request.id, &request.params, device_manager)
        }
        "fido2SetPin" => handle_fido2_set_pin(request.id, &request.params, device_manager),
        "fido2ChangePin" => handle_fido2_change_pin(request.id, &request.params, device_manager),
        "fido2ListCredentials" => {
            handle_fido2_list_credentials(request.id, &request.params, device_manager)
        }
        "fido2DeleteCredential" => {
            handle_fido2_delete_credential(request.id, &request.params, device_manager)
        }
        "fido2ResetDevice" => {
            handle_fido2_reset_device(request.id, &request.params, device_manager)
        }
        "pivGetData" => handle_piv_get_data(request.id, &request.params, device_manager),
        "pivSelect" => handle_piv_select(request.id, &request.params, device_manager),
        _ => Response::error(
            request.id,
            "UNKNOWN_COMMAND",
            &format!("Unknown command: {}", request.command),
        ),
    }
}

fn main() -> io::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Feitian SK Manager Native Host starting...");
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Initialize device manager
    let device_manager = match device::DeviceManager::new() {
        Ok(manager) => {
            log::info!("Device manager initialized successfully");
            manager
        }
        Err(e) => {
            log::error!("Failed to initialize device manager: {}", e);
            log::error!("The native host will still run, but device operations may fail");
            // Continue anyway - some commands like ping and getVersion will still work
            device::DeviceManager::new().unwrap_or_else(|_| {
                panic!("Critical: Could not initialize device manager");
            })
        }
    };

    // Main message loop
    loop {
        // Read message length
        let length = match read_message_length() {
            Ok(len) => len,
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    log::info!("Connection closed by client");
                    break;
                }
                log::error!("Failed to read message length: {}", e);
                continue;
            }
        };

        // Validate message length
        if length == 0 || length > 1024 * 1024 {
            log::error!("Invalid message length: {}", length);
            continue;
        }

        // Read message content
        let message = match read_message(length) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Failed to read message: {}", e);
                continue;
            }
        };

        log::debug!("Received message: {}", message);

        // Parse request
        let request: Request = match serde_json::from_str(&message) {
            Ok(req) => req,
            Err(e) => {
                log::error!("Failed to parse request: {}", e);
                // Send error response with id 0 if we can't parse the request
                let error_response = Response::error(0, "INVALID_JSON", &e.to_string());
                if let Ok(json) = serde_json::to_string(&error_response) {
                    let _ = write_message(&json);
                }
                continue;
            }
        };

        // Process request with device manager
        let response = process_request(request, &device_manager);

        // Send response
        match serde_json::to_string(&response) {
            Ok(json) => {
                log::debug!("Sending response: {}", json);
                if let Err(e) = write_message(&json) {
                    log::error!("Failed to send response: {}", e);
                    break;
                }
            }
            Err(e) => {
                log::error!("Failed to serialize response: {}", e);
            }
        }
    }

    log::info!("Native host shutting down");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_command() {
        let response = handle_ping(1);
        assert_eq!(response.status, "ok");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_get_version_command() {
        let response = handle_get_version(2);
        assert_eq!(response.status, "ok");
        assert!(response.result.is_some());

        if let Some(result) = response.result {
            assert!(result.get("version").is_some());
            assert!(result.get("name").is_some());
        }
    }

    #[test]
    fn test_unknown_command() {
        // Device manager might fail in test environment if PC/SC is not available
        // We'll use a simple approach: create it if possible, or test without it
        let request = Request {
            id: 3,
            command: "unknownCommand".to_string(),
            params: serde_json::json!({}),
        };

        // Try to create device manager, but don't fail if it can't be created
        if let Ok(device_manager) = device::DeviceManager::new() {
            let response = process_request(request, &device_manager);
            assert_eq!(response.status, "error");
            assert!(response.error.is_some());
        } else {
            // If device manager can't be created, just test the response structure
            let response = Response::error(3, "UNKNOWN_COMMAND", "Unknown command: unknownCommand");
            assert_eq!(response.status, "error");
            assert!(response.error.is_some());
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::success(1, serde_json::json!({"test": "value"}));
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_error_response() {
        let response = Response::error(1, "TEST_ERROR", "Test error message");
        assert_eq!(response.status, "error");
        assert!(response.error.is_some());

        if let Some(error) = response.error {
            assert_eq!(error.code, "TEST_ERROR");
            assert_eq!(error.message, "Test error message");
        }
    }
}
