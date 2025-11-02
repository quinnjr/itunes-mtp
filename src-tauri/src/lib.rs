// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod mtp;
mod app_state;
mod errors;

use std::sync::Mutex;
use tauri::State;

use mtp::{DeviceInfo, FileInfo, MtpDevice, ThreadSafeMtpManager};
use app_state::{AppState as LibraryState, ITunesLibrary, Track, Playlist};

// Application state
struct AppState {
    mtp_manager: ThreadSafeMtpManager,
    active_device: Mutex<Option<String>>,
    library_state: Mutex<Option<LibraryState>>,
}

impl AppState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            mtp_manager: ThreadSafeMtpManager::new()?,
            active_device: Mutex::new(None),
            library_state: Mutex::new(None),
        })
    }
}

// Tauri commands
#[tauri::command]
fn get_devices(state: State<AppState>) -> Result<Vec<DeviceInfo>, String> {
    state.mtp_manager
        .get_devices()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn connect_device(state: State<AppState>, device_id: String) -> Result<String, String> {
    // Test connection by creating a device instance
    let device = MtpDevice::new(&device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

    // Store the active device ID
    let mut active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    *active = Some(device_id.clone());

    Ok(format!("Connected to device: {}", device.get_device_id()))
}

#[tauri::command]
fn disconnect_device(state: State<AppState>) -> Result<String, String> {
    let mut active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    *active = None;
    Ok("Device disconnected".to_string())
}

#[tauri::command]
fn list_device_files(
    state: State<AppState>,
    folder_id: Option<String>,
) -> Result<Vec<FileInfo>, String> {
    let active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let device_id = active.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    let device = MtpDevice::new(device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

    device.list_files(folder_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_file_info(
    state: State<AppState>,
    object_id: String,
) -> Result<FileInfo, String> {
    let active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let device_id = active.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    let device = MtpDevice::new(device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

    device.get_file_info(&object_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn transfer_file(
    state: State<AppState>,
    object_id: String,
    dest_path: String,
) -> Result<String, String> {
    let active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let device_id = active.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    let device = MtpDevice::new(device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

    device.transfer_file(&object_id, &dest_path)
        .map_err(|e| e.to_string())?;

    Ok(format!("File transferred successfully to: {}", dest_path))
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
fn sync_playlist_to_device(
    state: State<AppState>,
    playlist_name: String,
    device_folder: Option<String>,
) -> Result<String, String> {
    let active = state.active_device.lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let device_id = active.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    let _device = MtpDevice::new(device_id)
        .map_err(|e| format!("Failed to connect to device: {}", e))?;

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
mod tests {
    use super::*;

    #[test]
    fn test_mtp_manager_creation() {
        // Test that we can create an MTP manager
        let result = ThreadSafeMtpManager::new();
        assert!(result.is_ok(), "Failed to create MTP manager: {:?}", result.err());
    }

    #[test]
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
            let device = MtpDevice::new(&device_info.device_id);
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
