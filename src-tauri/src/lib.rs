// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg(windows)]
mod mtp;
mod app_state;
mod errors;

use std::sync::Mutex;
use tauri::State;

#[cfg(windows)]
use mtp::{DeviceInfo, FileInfo, ThreadSafeMtpManager, ThreadSafeMtpDevice};
#[cfg(not(windows))]
mod mtp {
    use serde::Serialize;
    #[derive(Debug, Clone, Serialize)]
    pub struct DeviceInfo {
        pub device_id: String,
        pub friendly_name: String,
        pub manufacturer: String,
    }
    #[derive(Debug, Clone, Serialize)]
    pub struct FileInfo {
        pub object_id: String,
        pub name: String,
        pub size: u64,
        pub is_folder: bool,
    }
    pub struct MtpDevice;
    pub struct ThreadSafeMtpManager;
    impl ThreadSafeMtpManager {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            Err("MTP support only available on Windows".into())
        }
    }
}
#[cfg(not(windows))]
use mtp::{DeviceInfo, FileInfo};
use app_state::{AppState as LibraryState, ITunesLibrary, Track, Playlist};

// Application state
struct AppState {
    #[cfg(windows)]
    mtp_manager: ThreadSafeMtpManager,
    #[cfg(windows)]
    active_device_connection: Mutex<Option<ThreadSafeMtpDevice>>,
    active_device: Mutex<Option<String>>,
    library_state: Mutex<Option<LibraryState>>,
}

impl AppState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            #[cfg(windows)]
            mtp_manager: ThreadSafeMtpManager::new()?,
            #[cfg(windows)]
            active_device_connection: Mutex::new(None),
            active_device: Mutex::new(None),
            library_state: Mutex::new(None),
        })
    }
}

// Tauri commands
#[tauri::command]
#[cfg(windows)]
fn get_devices(state: State<AppState>) -> Result<Vec<DeviceInfo>, String> {
    state.mtp_manager
        .get_devices()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn get_devices(_state: State<AppState>) -> Result<Vec<()>, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn connect_device(state: State<AppState>, device_id: String) -> Result<String, String> {
    // Disconnect any existing device first
    let mut connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;
    if connection.is_some() {
        *connection = None;
    }
    drop(connection);

    // Create a persistent device connection
    let device = ThreadSafeMtpDevice::new(&device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

    // Store the active device connection and ID
    let mut connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;
    let mut active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    
    *connection = Some(device.clone());
    *active = Some(device_id.clone());

    Ok(format!("Connected to device: {}", device.get_device_id()))
}

#[tauri::command]
#[cfg(not(windows))]
fn connect_device(_state: State<AppState>, _device_id: String) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn disconnect_device(state: State<AppState>) -> Result<String, String> {
    // Clear the connection (device will be closed when dropped)
    let mut connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;
    let mut active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    
    *connection = None;
    *active = None;
    
    Ok("Device disconnected".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn disconnect_device(_state: State<AppState>) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn list_device_files(
    state: State<AppState>,
    folder_id: Option<String>,
) -> Result<Vec<FileInfo>, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    // Check if connection is still valid
    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.list_files(folder_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn list_device_files(_state: State<AppState>, _folder_id: Option<String>) -> Result<Vec<FileInfo>, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn get_file_info(
    state: State<AppState>,
    object_id: String,
) -> Result<FileInfo, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    // Check if connection is still valid
    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.get_file_info(&object_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn get_file_info(_state: State<AppState>, _object_id: String) -> Result<FileInfo, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn transfer_file(
    state: State<AppState>,
    object_id: String,
    dest_path: String,
) -> Result<String, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    // Check if connection is still valid
    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.transfer_file(&object_id, &dest_path)
        .map_err(|e| e.to_string())?;

    Ok(format!("File transferred successfully to: {}", dest_path))
}

#[tauri::command]
#[cfg(not(windows))]
fn transfer_file(_state: State<AppState>, _object_id: String, _dest_path: String) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
fn get_active_device(state: State<AppState>) -> Result<Option<String>, String> {
    let active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    Ok(active.clone())
}

// iTunes Library Commands
#[tauri::command]
fn parse_itunes_library(state: State<AppState>, xml_content: String) -> Result<ITunesLibrary, String> {
    let mut library_state = LibraryState::default();
    let temp_path = std::env::temp_dir().join("temp_library.xml");

    // Write content to temp file
    std::fs::write(&temp_path, xml_content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Parse library
    let library = library_state.parse_library(&temp_path)
        .map_err(|e| format!("Failed to parse library: {}", e))?;

    // Store library state
    let mut lib_state = state.library_state.lock()
        .map_err(|e| format!("Failed to lock library state: {}", e))?;
    *lib_state = Some(library_state);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(library)
}

#[tauri::command]
fn get_playlists(state: State<AppState>) -> Result<Vec<Playlist>, String> {
    let lib_state = state.library_state.lock()
        .map_err(|e| format!("Failed to lock library state: {}", e))?;

    if let Some(library) = lib_state.as_ref() {
        if let Some(lib) = &library.library {
            Ok(lib.playlists.clone())
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn get_tracks(state: State<AppState>) -> Result<Vec<Track>, String> {
    let lib_state = state.library_state.lock()
        .map_err(|e| format!("Failed to lock library state: {}", e))?;

    if let Some(library) = lib_state.as_ref() {
        if let Some(lib) = &library.library {
            Ok(lib.tracks.values().cloned().collect::<Vec<_>>())
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
#[cfg(windows)]
fn sync_playlist_to_device(
    state: State<AppState>,
    playlist_name: String,
    device_folder: Option<String>,
) -> Result<String, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    // Check if connection is still valid
    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    // Get library state
    let lib_state = state.library_state.lock()
        .map_err(|e| format!("Failed to lock library state: {}", e))?;

    if let Some(library) = lib_state.as_ref() {
        if let Some(lib) = &library.library {
            // Find the playlist
            if let Some(playlist) = lib.playlists.iter().find(|p| p.name == playlist_name) {
                // Create M3U content
                let _m3u_content = library.generate_mtp_playlist_content(playlist)
                    .map_err(|e| format!("Failed to generate playlist content: {}", e))?;

                // For now, just return success - in a real implementation,
                // you would transfer files and create the playlist on the device
                Ok(format!("Playlist '{}' would be synced to device folder: {:?}",
                    playlist_name, device_folder.unwrap_or_else(|| "Music".to_string())))
            } else {
                Err(format!("Playlist '{}' not found", playlist_name))
            }
        } else {
            Err("No library loaded".to_string())
        }
    } else {
        Err("No library loaded".to_string())
    }
}

#[tauri::command]
#[cfg(not(windows))]
fn sync_playlist_to_device(_state: State<AppState>, _playlist_name: String, _device_folder: Option<String>) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new()
        .expect("Failed to initialize application state");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_devices,
            connect_device,
            disconnect_device,
            list_device_files,
            get_file_info,
            transfer_file,
            get_active_device,
            parse_itunes_library,
            get_playlists,
            get_tracks,
            sync_playlist_to_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Ignore in CI to avoid COM initialization crashes
    fn test_mtp_manager_creation() {
        // Test that we can create an MTP manager
        let result = ThreadSafeMtpManager::new();
        assert!(result.is_ok(), "Failed to create MTP manager: {:?}", result.err());
    }

    #[test]
    #[ignore] // Ignore in CI to avoid COM initialization crashes
    fn test_get_devices() {
        // Test that we can enumerate devices (may be empty if no devices connected)
        let manager = ThreadSafeMtpManager::new()
            .expect("Failed to create MTP manager");

        let result = manager.get_devices();
        assert!(result.is_ok(), "Failed to get devices: {:?}", result.err());

        let devices = result.unwrap();
        println!("Found {} MTP device(s)", devices.len());
        for device in devices {
            println!("  - {} ({})", device.friendly_name, device.manufacturer);
        }
    }

    #[test]
    #[ignore] // Only run when a device is connected
    fn test_device_connection() {
        let manager = ThreadSafeMtpManager::new()
            .expect("Failed to create MTP manager");

        let devices = manager.get_devices()
            .expect("Failed to get devices");

        if let Some(device_info) = devices.first() {
            let device = ThreadSafeMtpDevice::new(&device_info.device_id);
            assert!(device.is_ok(), "Failed to connect to device: {:?}", device.err());

            if let Ok(device) = device {
                let files = device.list_files(None);
                assert!(files.is_ok(), "Failed to list files: {:?}", files.err());

                if let Ok(files) = files {
                    println!("Found {} object(s) in device root", files.len());
                    for file in files.iter().take(5) {
                        println!("  - {} ({} bytes, folder: {})",
                            file.name, file.size, file.is_folder);
                    }
                }
            }
        } else {
            println!("No devices found for testing");
        }
    }
}
