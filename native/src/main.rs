use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Request structure for JSON-RPC messages
#[derive(Debug, Deserialize)]
struct Request {
    id: u32,
    command: String,
    #[serde(default)]
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

/// Process a single request
fn process_request(request: Request) -> Response {
    log::info!(
        "Processing command: {} (id: {})",
        request.command,
        request.id
    );

    match request.command.as_str() {
        "ping" => handle_ping(request.id),
        "getVersion" => handle_get_version(request.id),
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

        // Process request
        let response = process_request(request);

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
        let request = Request {
            id: 3,
            command: "unknownCommand".to_string(),
            params: serde_json::json!({}),
        };
        let response = process_request(request);
        assert_eq!(response.status, "error");
        assert!(response.error.is_some());
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
