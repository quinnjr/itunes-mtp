use std::{
    error::Error,
    io::Write,
    result::Result,
    vec::Vec,
};

use windows::{
    core::*,
    Win32::Devices::PortableDevices::*,
    Win32::System::Com::*,
};

#[derive(Debug)]
pub struct DeviceInfo {
    pub device_id: String,
    pub friendly_name: String,
    pub manufacturer: String,
}

#[derive(Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub content_type: GUID,
}

pub struct MtpDeviceManager {
    device_manager: IPortableDeviceManager,
}

impl MtpDeviceManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unsafe {
            // Initialize COM
            CoInitializeEx(None, COINIT_MULTITHREADED)?;

            // Create device manager instance
            let device_manager: IPortableDeviceManager = CoCreateInstance(
                &PortableDeviceManager as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            )?;

            Ok(Self { device_manager })
        }
    }

    pub fn get_devices(&self) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        unsafe {
            let mut device_count: u32 = 0;
            self.device_manager.GetDevices(std::ptr::null_mut(), &mut device_count)?;

            let mut device_ids: Vec<PWSTR> = vec![PWSTR::null(); device_count as usize];
            self.device_manager.GetDevices(device_ids.as_mut_ptr(), &mut device_count)?;

            let mut devices = Vec::new();
            for device_id in device_ids.into_iter().take(device_count as usize) {
                if let Ok(device_id_str) = device_id.to_string() {
                    let device_id_pcwstr = PCWSTR::from_raw(device_id.as_ptr());

                    // Friendly name
                    let mut friendly_name_len = 0;
                    self.device_manager.GetDeviceFriendlyName(device_id_pcwstr, PWSTR::null(), &mut friendly_name_len)?;
                    let mut friendly_buf = vec![0u16; friendly_name_len as usize];
                    self.device_manager.GetDeviceFriendlyName(device_id_pcwstr, PWSTR(friendly_buf.as_mut_ptr()), &mut friendly_name_len)?;
                    let friendly_name = String::from_utf16_lossy(&friendly_buf[..friendly_name_len as usize - 1]);

                    // Manufacturer
                    let mut manufacturer_len = 0;
                    self.device_manager.GetDeviceManufacturer(device_id_pcwstr, PWSTR::null(), &mut manufacturer_len)?;
                    let mut manufacturer_buf = vec![0u16; manufacturer_len as usize];
                    self.device_manager.GetDeviceManufacturer(device_id_pcwstr, PWSTR(manufacturer_buf.as_mut_ptr()), &mut manufacturer_len)?;
                    let manufacturer = String::from_utf16_lossy(&manufacturer_buf[..manufacturer_len as usize - 1]);

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
}

impl Drop for MtpDeviceManager {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub struct MtpDevice {
    device: IPortableDevice,
    content: IPortableDeviceContent,
}

#[derive(Debug)]
pub struct MtpError(String);

impl std::fmt::Display for MtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MTP error: {}", self.0)
    }
}

impl Error for MtpError {}

impl MtpDevice {
    pub fn new(device_id: &str) -> Result<Self, Box<dyn Error>> {
        unsafe {
            // Initialize COM
            CoInitializeEx(None, COINIT_MULTITHREADED)?;

            // Create device instance
            let device: IPortableDevice = CoCreateInstance(
                &PortableDevice as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            )?;

            // Open device
            let client_info: IPortableDeviceValues = CoCreateInstance(
                &PortableDeviceValues as *const GUID,
                None,
                CLSCTX_INPROC_SERVER,
            )?;
            device.Open(PCWSTR::from_raw(device_id.as_ptr() as *const u16), &client_info)?;

            // Get content interface
            let content: IPortableDeviceContent = device.Content()?;

            Ok(Self { device, content })
        }
    }

    pub fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<String>, Box<dyn Error>> {
        unsafe {
            let enum_objects = self.content.EnumObjects(
                0,
                folder_id.map_or(PCWSTR::null(), |id| PCWSTR::from_raw(id.as_ptr() as *const u16)),
                None,
            )?;

            let mut files = Vec::new();
            let mut fetched = 0;
            let mut object_ids = [PWSTR::null(); 10];

            while enum_objects.Next(
                &mut object_ids,
                &mut fetched,
            ).is_ok() && fetched > 0 {
                for i in 0..fetched as usize {
                    if let Ok(id) = object_ids[i].to_string() {
                        files.push(id);
                    }
                }
            }

            Ok(files)
        }
    }

    pub fn get_file_info(&self, file_id: &str) -> Result<FileInfo, Box<dyn Error>> {
        unsafe {
            let properties = self.content.Properties()?;
            let object_properties = properties.GetValues(
                PCWSTR::from_raw(file_id.as_ptr() as *const u16),
                None,
            )?;

            // Retrieve name
            let name_variant = object_properties.GetValue(&WPD_OBJECT_NAME)?;
            let name = name_variant.Anonymous.Anonymous.Anonymous.pwszVal.to_string()?;

            // Retrieve size
            let size_variant = object_properties.GetValue(&WPD_OBJECT_SIZE)?;
            // Remove ".QuadPart" access, convert directly to u64.
            let size = size_variant.Anonymous.Anonymous.Anonymous.uhVal as u64;

            // Retrieve content_type
            let content_type_variant = object_properties.GetValue(&WPD_OBJECT_CONTENT_TYPE)?;
            let content_type = *content_type_variant.Anonymous.Anonymous.Anonymous.puuid;

            Ok(FileInfo {
                name,
                size,
                content_type,
            })
        }
    }

    pub fn transfer_file(&self, file_id: &str, dest_path: &str) -> Result<(), Box<dyn Error>> {
        unsafe {
            let resources = self.content.Transfer()?;

            let chunk_size = 64 * 1024;
            let mut buffer = vec![0u8; chunk_size];

            let mut optimal_buffer_size: u32 = 0;
            let mut stream: Option<IStream> = None;
            resources.GetStream(
                PCWSTR::from_raw(file_id.as_ptr() as *const u16),
                &WPD_RESOURCE_DEFAULT,
                STGM_READ.0 as u32,
                &mut optimal_buffer_size,
                &mut stream,
            )?;

            let resource = stream.ok_or("No resource returned")?;
            let mut file = std::fs::File::create(dest_path)?;

            loop {
                let mut read: u32 = 0;
                // Pass a pointer to the buffer, plus its size, plus a pointer to read
                let hr = resource.Read(
                    buffer.as_mut_ptr() as *mut core::ffi::c_void,
                    buffer.len() as u32,
                    Some(&mut read),
                );

                // If read fails or returns 0 bytes, break
                if hr.is_err() || read == 0 {
                    break;
                }

                file.write_all(&buffer[..read as usize])?;
            }

            Ok(())
        }
    }
}

impl Drop for MtpDevice {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}