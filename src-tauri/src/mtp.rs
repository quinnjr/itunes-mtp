#[cfg(windows)]
use std::{
    error::Error,
    fmt,
    io::Write,
    result::Result,
    sync::{Arc, Mutex},
};

#[cfg(windows)]
use windows::{
    core::*,
    Win32::Devices::PortableDevices::*,
    Win32::System::Com::*,
    Win32::System::Com::StructuredStorage::{PropVariantToStringAlloc, PropVariantToUInt64, PropVariantToGUID},
    Win32::Foundation::RPC_E_CHANGED_MODE,
};

#[cfg(windows)]
use serde::Serialize;

// Custom error types
#[cfg(windows)]
#[derive(Debug)]
pub enum MtpError {
    ComError(String),
    DeviceError(String),
    NotFound(String),
    TransferError(String),
    InvalidOperation(String),
}

#[cfg(windows)]
impl fmt::Display for MtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MtpError::ComError(msg) => write!(f, "COM error: {}", msg),
            MtpError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            MtpError::NotFound(msg) => write!(f, "Not found: {}", msg),
            MtpError::TransferError(msg) => write!(f, "Transfer error: {}", msg),
            MtpError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

#[cfg(windows)]
impl Error for MtpError {}

#[cfg(windows)]
unsafe impl Send for MtpError {}
#[cfg(windows)]
unsafe impl Sync for MtpError {}

// Device information
#[cfg(windows)]
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub friendly_name: String,
    pub manufacturer: String,
}

// File information
#[cfg(windows)]
#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub object_id: String,
    pub name: String,
    pub size: u64,
    pub is_folder: bool,
}

// MTP Device Manager
#[cfg(windows)]
pub struct MtpDeviceManager {
    device_manager: IPortableDeviceManager,
    _com_initialized: bool,
}

#[cfg(windows)]
impl MtpDeviceManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unsafe {
            // Initialize COM
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            let com_initialized = hr.is_ok() || hr == RPC_E_CHANGED_MODE;

            // Create device manager instance
            let device_manager: IPortableDeviceManager = CoCreateInstance(
                &PortableDeviceManager as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            ).map_err(|e| MtpError::ComError(format!("Failed to create device manager: {}", e)))?;

            Ok(Self {
                device_manager,
                _com_initialized: com_initialized,
            })
        }
    }

    pub fn get_devices(&self) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        unsafe {
            // Get device count
            let mut device_count: u32 = 0;
            self.device_manager
                .GetDevices(std::ptr::null_mut(), &mut device_count)
                .map_err(|e| MtpError::DeviceError(format!("Failed to get device count: {}", e)))?;

            if device_count == 0 {
                return Ok(Vec::new());
            }

            // Get device IDs
            let mut device_ids: Vec<PWSTR> = vec![PWSTR::null(); device_count as usize];
            self.device_manager
                .GetDevices(device_ids.as_mut_ptr(), &mut device_count)
                .map_err(|e| MtpError::DeviceError(format!("Failed to enumerate devices: {}", e)))?;

            let mut devices = Vec::new();

            for device_id in device_ids.into_iter().take(device_count as usize) {
                if let Ok(device_id_str) = device_id.to_string() {
                    let device_id_pcwstr = PCWSTR::from_raw(device_id.as_ptr());

                    // Get friendly name
                    let friendly_name = self.get_device_string(
                        |buf, len| self.device_manager.GetDeviceFriendlyName(device_id_pcwstr, buf, len)
                    ).unwrap_or_else(|_| "Unknown Device".to_string());

                    // Get manufacturer
                    let manufacturer = self.get_device_string(
                        |buf, len| self.device_manager.GetDeviceManufacturer(device_id_pcwstr, buf, len)
                    ).unwrap_or_else(|_| "Unknown Manufacturer".to_string());

                    devices.push(DeviceInfo {
                        device_id: device_id_str,
                        friendly_name,
                        manufacturer,
                    });
                }
            }

            Ok(devices)
        }
    }

    unsafe fn get_device_string<F>(&self, mut getter: F) -> Result<String, Box<dyn Error>>
    where
        F: FnMut(PWSTR, *mut u32) -> windows::core::Result<()>,
    {
        let mut len = 0;
        getter(PWSTR::null(), &mut len)?;

        if len == 0 {
            return Ok(String::new());
        }

        let mut buffer = vec![0u16; len as usize];
        getter(PWSTR(buffer.as_mut_ptr()), &mut len)?;

        Ok(String::from_utf16_lossy(&buffer[..len.saturating_sub(1) as usize]))
    }
}

#[cfg(windows)]
impl Drop for MtpDeviceManager {
    fn drop(&mut self) {
        if self._com_initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

#[cfg(windows)]
unsafe impl Send for MtpDeviceManager {}
#[cfg(windows)]
unsafe impl Sync for MtpDeviceManager {}

// MTP Device
#[cfg(windows)]
pub struct MtpDevice {
    device: IPortableDevice,
    content: IPortableDeviceContent,
    device_id: String,
    _com_initialized: bool,
}

#[cfg(windows)]
impl MtpDevice {
    pub fn new(device_id: &str) -> Result<Self, Box<dyn Error>> {
        unsafe {
            // Initialize COM
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            let com_initialized = hr.is_ok() || hr == RPC_E_CHANGED_MODE;

            // Create device instance
            let device: IPortableDevice = CoCreateInstance(
                &PortableDevice as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            ).map_err(|e| MtpError::ComError(format!("Failed to create device instance: {}", e)))?;

            // Create client info
            let client_info: IPortableDeviceValues = CoCreateInstance(
                &PortableDeviceValues as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            ).map_err(|e| MtpError::ComError(format!("Failed to create client info: {}", e)))?;

            // Set client information
            client_info.SetStringValue(&WPD_CLIENT_NAME, &HSTRING::from("iTunes MTP Sync"))?;
            client_info.SetUnsignedIntegerValue(&WPD_CLIENT_MAJOR_VERSION, 1)?;
            client_info.SetUnsignedIntegerValue(&WPD_CLIENT_MINOR_VERSION, 0)?;
            client_info.SetUnsignedIntegerValue(&WPD_CLIENT_REVISION, 0)?;

            // Convert device_id to wide string
            let device_id_wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
            let device_id_pcwstr = PCWSTR::from_raw(device_id_wide.as_ptr());

            // Open device
            device.Open(device_id_pcwstr, &client_info)
                .map_err(|e| MtpError::DeviceError(format!("Failed to open device: {}", e)))?;

            // Get content interface
            let content = device.Content()
                .map_err(|e| MtpError::DeviceError(format!("Failed to get content interface: {}", e)))?;

            Ok(Self {
                device,
                content,
                device_id: device_id.to_string(),
                _com_initialized: com_initialized,
            })
        }
    }

    pub fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<FileInfo>, Box<dyn Error>> {
        unsafe {
            let parent_id = if let Some(id) = folder_id {
                let id_wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
                PCWSTR::from_raw(id_wide.as_ptr())
            } else {
                WPD_DEVICE_OBJECT_ID
            };

            let enum_objects = self.content
                .EnumObjects(0, parent_id, None)
                .map_err(|e| MtpError::DeviceError(format!("Failed to enumerate objects: {}", e)))?;

            let mut files = Vec::new();
            let mut object_ids = [PWSTR::null(); 10];
            let mut fetched = 0;

            loop {
                let result = enum_objects.Next(&mut object_ids, &mut fetched);

                if result.is_err() || fetched == 0 {
                    break;
                }

                for i in 0..fetched as usize {
                    let object_id_str = object_ids[i].to_string().unwrap_or_default();
                    if !object_id_str.is_empty() {
                        if let Ok(file_info) = self.get_file_info(&object_id_str) {
                            files.push(file_info);
                        }
                    }
                }
            }

            Ok(files)
        }
    }

    pub fn get_file_info(&self, object_id: &str) -> Result<FileInfo, Box<dyn Error>> {
        unsafe {
            let properties = self.content.Properties()
                .map_err(|e| MtpError::DeviceError(format!("Failed to get properties interface: {}", e)))?;

            let object_id_wide: Vec<u16> = object_id.encode_utf16().chain(std::iter::once(0)).collect();
            let object_id_pcwstr = PCWSTR::from_raw(object_id_wide.as_ptr());

            let object_properties = properties.GetValues(object_id_pcwstr, None)
                .map_err(|e| MtpError::DeviceError(format!("Failed to get object properties: {}", e)))?;

            // Get name
            let name = match object_properties.GetValue(&WPD_OBJECT_NAME) {
                Ok(variant) => PropVariantToStringAlloc(&variant)
                    .and_then(|s| Ok(s.to_string()))
                    .unwrap_or_else(|_| Ok("Unknown".to_string()))
                    .unwrap_or_else(|_| "Unknown".to_string()),
                Err(_) => "Unknown".to_string(),
            };

            // Get size
            let size = match object_properties.GetValue(&WPD_OBJECT_SIZE) {
                Ok(variant) => PropVariantToUInt64(&variant).unwrap_or(0),
                Err(_) => 0,
            };

            // Check if it's a folder
            let is_folder = match object_properties.GetValue(&WPD_OBJECT_CONTENT_TYPE) {
                Ok(variant) => {
                    if let Ok(guid) = PropVariantToGUID(&variant) {
                        guid == WPD_CONTENT_TYPE_FOLDER
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            Ok(FileInfo {
                object_id: object_id.to_string(),
                name,
                size,
                is_folder,
            })
        }
    }

    pub fn transfer_file(&self, object_id: &str, dest_path: &str) -> Result<(), Box<dyn Error>> {
        unsafe {
            let resources = self.content.Transfer()
                .map_err(|e| MtpError::TransferError(format!("Failed to get transfer interface: {}", e)))?;

            let object_id_wide: Vec<u16> = object_id.encode_utf16().chain(std::iter::once(0)).collect();
            let object_id_pcwstr = PCWSTR::from_raw(object_id_wide.as_ptr());

            let mut optimal_buffer_size: u32 = 0;
            let mut stream: Option<IStream> = None;

            resources.GetStream(
                object_id_pcwstr,
                &WPD_RESOURCE_DEFAULT,
                STGM_READ.0 as u32,
                &mut optimal_buffer_size,
                &mut stream,
            ).map_err(|e| MtpError::TransferError(format!("Failed to get stream: {}", e)))?;

            let stream = stream.ok_or_else(|| MtpError::TransferError("No stream returned".to_string()))?;

            let buffer_size = if optimal_buffer_size > 0 {
                optimal_buffer_size as usize
            } else {
                64 * 1024
            };

            let mut buffer = vec![0u8; buffer_size];
            let mut file = std::fs::File::create(dest_path)
                .map_err(|e| MtpError::TransferError(format!("Failed to create destination file: {}", e)))?;

            loop {
                let mut bytes_read: u32 = 0;
                let hr = stream.Read(
                    buffer.as_mut_ptr() as *mut core::ffi::c_void,
                    buffer.len() as u32,
                    Some(&mut bytes_read),
                );

                if hr.is_err() || bytes_read == 0 {
                    break;
                }

                file.write_all(&buffer[..bytes_read as usize])
                    .map_err(|e| MtpError::TransferError(format!("Failed to write to file: {}", e)))?;
            }

            Ok(())
        }
    }

    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
}

#[cfg(windows)]
impl Drop for MtpDevice {
    fn drop(&mut self) {
        unsafe {
            // Close the device
            let _ = self.device.Close();
        }

        if self._com_initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

#[cfg(windows)]
unsafe impl Send for MtpDevice {}
#[cfg(windows)]
unsafe impl Sync for MtpDevice {}

// Thread-safe wrapper
#[cfg(windows)]
pub struct ThreadSafeMtpManager {
    manager: Arc<Mutex<MtpDeviceManager>>,
}

#[cfg(windows)]
impl ThreadSafeMtpManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let manager = MtpDeviceManager::new()?;
        Ok(Self {
            manager: Arc::new(Mutex::new(manager)),
        })
    }

    pub fn get_devices(&self) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        let manager = self.manager.lock()
            .map_err(|e| MtpError::InvalidOperation(format!("Failed to lock manager: {}", e)))?;
        manager.get_devices()
    }
}

#[cfg(windows)]
impl Clone for ThreadSafeMtpManager {
    fn clone(&self) -> Self {
        Self {
            manager: Arc::clone(&self.manager),
        }
    }
}
