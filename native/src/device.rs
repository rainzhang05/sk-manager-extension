use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feitian Technologies Vendor ID
const FEITIAN_VENDOR_ID: u16 = 0x096e;

/// Device type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum DeviceType {
    Hid,
    Ccid,
}

/// Device information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_type: DeviceType,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub path: String,
}

/// Enumerate HID devices and filter by Feitian vendor ID
fn enumerate_hid_devices() -> Result<Vec<Device>> {
    log::debug!("Enumerating HID devices...");

    let api = hidapi::HidApi::new().context("Failed to initialize HID API")?;

    let mut devices = Vec::new();
    let mut device_counter = 0;

    for device_info in api.device_list() {
        // Filter by Feitian vendor ID
        if device_info.vendor_id() != FEITIAN_VENDOR_ID {
            continue;
        }

        // Get HID usage page and usage
        let usage_page = device_info.usage_page();
        let usage = device_info.usage();

        log::debug!(
            "HID device - Path: {}, Usage Page: 0x{:04x}, Usage: 0x{:04x}",
            device_info.path().to_string_lossy(),
            usage_page,
            usage
        );

        // Skip obvious non-FIDO interfaces (keyboard=0x01/0x06, mouse=0x01/0x02)
        // But keep everything else including unknown usage pages
        if usage_page == 0x01 && (usage == 0x02 || usage == 0x06) {
            log::debug!("Skipping keyboard/mouse interface (usage page 0x{:04x}, usage 0x{:04x})", usage_page, usage);
            continue;
        }

        device_counter += 1;

        let manufacturer = device_info.manufacturer_string().map(|s| s.to_string());
        let product_name = device_info.product_string().map(|s| s.to_string());
        let serial_number = device_info.serial_number().map(|s| s.to_string());

        // Generate unique ID based on path or sequential number
        let id = format!("hid_{}", device_counter);

        let device = Device {
            id: id.clone(),
            vendor_id: device_info.vendor_id(),
            product_id: device_info.product_id(),
            device_type: DeviceType::Hid,
            manufacturer,
            product_name,
            serial_number,
            path: device_info.path().to_string_lossy().to_string(),
        };

        log::info!(
            "Found HID device: {} - VID: 0x{:04x}, PID: 0x{:04x}, Usage Page: 0x{:04x}, Usage: 0x{:04x}, Path: {}",
            device
                .product_name
                .as_ref()
                .unwrap_or(&"Unknown".to_string()),
            device.vendor_id,
            device.product_id,
            usage_page,
            usage,
            device.path
        );

        devices.push(device);
    }

    log::debug!("Found {} HID devices with Feitian VID", devices.len());
    Ok(devices)
}

/// Enumerate CCID readers and filter for Feitian devices
fn enumerate_ccid_devices() -> Result<Vec<Device>> {
    log::debug!("Enumerating CCID readers...");

    // Try to establish PC/SC context
    let ctx = match pcsc::Context::establish(pcsc::Scope::User) {
        Ok(ctx) => ctx,
        Err(e) => {
            log::warn!(
                "Failed to establish PC/SC context: {}. Skipping CCID enumeration.",
                e
            );
            return Ok(Vec::new());
        }
    };

    // Get list of readers
    let readers_buf = match ctx.list_readers_owned() {
        Ok(buf) => buf,
        Err(e) => {
            log::warn!(
                "Failed to list PC/SC readers: {}. No CCID devices found.",
                e
            );
            return Ok(Vec::new());
        }
    };

    let mut devices = Vec::new();
    let mut device_counter = 0;

    for reader_name in readers_buf.iter() {
        let reader_str = reader_name.to_string_lossy();
        log::debug!("Checking reader: {}", reader_str);

        // Check if this is a Feitian reader
        // Common Feitian reader names contain "Feitian", "ePass", "BioPass", etc.
        let is_feitian = reader_str.to_lowercase().contains("feitian")
            || reader_str.to_lowercase().contains("epass")
            || reader_str.to_lowercase().contains("biopass");

        if !is_feitian {
            log::debug!("Skipping non-Feitian reader: {}", reader_str);
            continue;
        }

        device_counter += 1;

        // Try to connect to the card to get more info
        let (manufacturer, product_name, serial_number) =
            match ctx.connect(reader_name, pcsc::ShareMode::Shared, pcsc::Protocols::ANY) {
                Ok(card) => {
                    // Try to get ATR (Answer To Reset) for device identification
                    match card.status2_owned() {
                        Ok(_status) => {
                            log::debug!("Card status retrieved for {}", reader_str);
                            // We could parse ATR here for more detailed info
                            // For now, we'll use the reader name as product name
                            (
                                Some("Feitian Technologies".to_string()),
                                Some(reader_str.to_string()),
                                None,
                            )
                        }
                        Err(e) => {
                            log::debug!("Could not get card status for {}: {}", reader_str, e);
                            (
                                Some("Feitian Technologies".to_string()),
                                Some(reader_str.to_string()),
                                None,
                            )
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Could not connect to card in {}: {}", reader_str, e);
                    (
                        Some("Feitian Technologies".to_string()),
                        Some(reader_str.to_string()),
                        None,
                    )
                }
            };

        let id = format!("ccid_{}", device_counter);

        let device = Device {
            id: id.clone(),
            vendor_id: FEITIAN_VENDOR_ID, // Assume Feitian VID
            product_id: 0,                // Unknown for CCID, would need ATR parsing
            device_type: DeviceType::Ccid,
            manufacturer,
            product_name,
            serial_number,
            path: reader_str.to_string(),
        };

        log::info!(
            "Found CCID device: {} - Reader: {}",
            device
                .product_name
                .as_ref()
                .unwrap_or(&"Unknown".to_string()),
            device.path
        );

        devices.push(device);
    }

    log::debug!("Found {} CCID devices", devices.len());
    Ok(devices)
}

/// List all Feitian devices (both HID and CCID)
pub fn list_devices() -> Result<Vec<Device>> {
    log::info!("Starting device enumeration...");

    let mut all_devices = Vec::new();
    let mut seen_paths = HashMap::new();

    // Enumerate HID devices
    match enumerate_hid_devices() {
        Ok(hid_devices) => {
            for device in hid_devices {
                seen_paths.insert(device.path.clone(), true);
                all_devices.push(device);
            }
        }
        Err(e) => {
            log::error!("Failed to enumerate HID devices: {}", e);
            // Continue with CCID enumeration even if HID fails
        }
    }

    // Enumerate CCID devices
    match enumerate_ccid_devices() {
        Ok(ccid_devices) => {
            for device in ccid_devices {
                // Avoid duplicates (some devices may appear as both HID and CCID)
                if !seen_paths.contains_key(&device.path) {
                    all_devices.push(device);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to enumerate CCID devices: {}", e);
            // Continue even if CCID fails
        }
    }

    log::info!("Total devices found: {}", all_devices.len());

    if all_devices.is_empty() {
        log::info!("No Feitian devices detected. Make sure your device is connected.");
    }

    Ok(all_devices)
}

/// Open device enum - represents an open HID or CCID device
pub enum OpenDevice {
    Hid(hidapi::HidDevice),
    Ccid(pcsc::Card),
}

/// Device manager with connection tracking
pub struct DeviceManager {
    hid_api: std::sync::Arc<std::sync::Mutex<hidapi::HidApi>>,
    pcsc_context: std::sync::Arc<std::sync::Mutex<pcsc::Context>>,
    open_devices: std::sync::Arc<std::sync::Mutex<HashMap<String, OpenDevice>>>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Result<Self> {
        let hid_api = hidapi::HidApi::new().context("Failed to initialize HID API")?;
        let pcsc_context = pcsc::Context::establish(pcsc::Scope::User)
            .context("Failed to establish PC/SC context")?;

        Ok(Self {
            hid_api: std::sync::Arc::new(std::sync::Mutex::new(hid_api)),
            pcsc_context: std::sync::Arc::new(std::sync::Mutex::new(pcsc_context)),
            open_devices: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Open a device by its ID
    pub fn open_device(&self, device_id: &str) -> Result<()> {
        let mut open_devices = self.open_devices.lock().unwrap();

        // Check if device is already open
        if open_devices.contains_key(device_id) {
            return Err(anyhow::anyhow!("Device {} is already open", device_id));
        }

        // Get all devices
        let all_devices = list_devices()?;
        let device = all_devices
            .iter()
            .find(|d| d.id == device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        log::info!("Opening device: {} ({:?})", device_id, device.device_type);

        // Open based on device type
        match device.device_type {
            DeviceType::Hid => {
                let hid_api = self.hid_api.lock().unwrap();

                // Always try to open by path first since we have the exact path from enumeration
                // This is more reliable than VID/PID when there are multiple interfaces
                let hid_device = match hid_api.open_path(&std::ffi::CString::new(device.path.as_bytes())?) {
                    Ok(dev) => {
                        log::debug!("Opened HID device by path: {}", device.path);
                        dev
                    }
                    Err(e) => {
                        log::debug!("Failed to open by path ({}), trying VID/PID as fallback", e);
                        // Fallback to VID/PID only if path fails
                        hid_api
                            .open(device.vendor_id, device.product_id)
                            .context(format!(
                                "Failed to open HID device. Tried path {} and VID/PID {:04x}:{:04x}. \
                                 Error: {}. The device may be in use by another application, or you may need to grant \
                                 permission in System Settings > Privacy & Security > Input Monitoring",
                                device.path,
                                device.vendor_id,
                                device.product_id,
                                e
                            ))?
                    }
                };

                open_devices.insert(device_id.to_string(), OpenDevice::Hid(hid_device));
                log::info!("Successfully opened HID device: {}", device_id);
            }
            DeviceType::Ccid => {
                let pcsc_context = self.pcsc_context.lock().unwrap();
                let reader_name = std::ffi::CString::new(device.path.as_bytes())
                    .context("Invalid reader name")?;

                let card = pcsc_context
                    .connect(&reader_name, pcsc::ShareMode::Shared, pcsc::Protocols::ANY)
                    .context(format!("Failed to connect to CCID card at {}", device.path))?;

                open_devices.insert(device_id.to_string(), OpenDevice::Ccid(card));
                log::info!("Successfully opened CCID card: {}", device_id);
            }
        }

        Ok(())
    }

    /// Close a device by its ID
    pub fn close_device(&self, device_id: &str) -> Result<()> {
        let mut open_devices = self.open_devices.lock().unwrap();

        if !open_devices.contains_key(device_id) {
            return Err(anyhow::anyhow!("Device {} is not open", device_id));
        }

        log::info!("Closing device: {}", device_id);
        open_devices.remove(device_id);
        log::info!("Successfully closed device: {}", device_id);

        Ok(())
    }

    /// Execute an operation with a HID device
    pub fn with_hid_device<F, R>(&self, device_id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&hidapi::HidDevice) -> Result<R>,
    {
        let open_devices = self.open_devices.lock().unwrap();

        match open_devices.get(device_id) {
            Some(OpenDevice::Hid(device)) => f(device),
            Some(OpenDevice::Ccid(_)) => Err(anyhow::anyhow!(
                "Device {} is a CCID device, not HID",
                device_id
            )),
            None => Err(anyhow::anyhow!("Device {} is not open", device_id)),
        }
    }

    /// Execute an operation with a CCID card
    pub fn with_ccid_card<F, R>(&self, device_id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&pcsc::Card) -> Result<R>,
    {
        let open_devices = self.open_devices.lock().unwrap();

        match open_devices.get(device_id) {
            Some(OpenDevice::Ccid(card)) => f(card),
            Some(OpenDevice::Hid(_)) => Err(anyhow::anyhow!(
                "Device {} is a HID device, not CCID",
                device_id
            )),
            None => Err(anyhow::anyhow!("Device {} is not open", device_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_serialization() {
        let device = Device {
            id: "test1".to_string(),
            vendor_id: FEITIAN_VENDOR_ID,
            product_id: 0x0852,
            device_type: DeviceType::Hid,
            manufacturer: Some("Feitian Technologies".to_string()),
            product_name: Some("ePass FIDO".to_string()),
            serial_number: Some("ABC123".to_string()),
            path: "/dev/hidraw0".to_string(),
        };

        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"vendor_id\":2414"));
        assert!(json.contains("\"device_type\":\"Hid\""));
    }

    #[test]
    fn test_list_devices_no_panic() {
        // This test should not panic even if no devices are connected
        let result = list_devices();
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_type_serialization() {
        let hid_type = DeviceType::Hid;
        let ccid_type = DeviceType::Ccid;

        let hid_json = serde_json::to_string(&hid_type).unwrap();
        let ccid_json = serde_json::to_string(&ccid_type).unwrap();

        assert_eq!(hid_json, "\"Hid\"");
        assert_eq!(ccid_json, "\"Ccid\"");
    }
}
