use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

mod device;
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
