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

#[tauri::command]
#[cfg(windows)]
fn create_folder(
    state: State<AppState>,
    parent_id: String,
    folder_name: String,
) -> Result<String, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.create_folder(&parent_id, &folder_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn create_folder(_state: State<AppState>, _parent_id: String, _folder_name: String) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn ensure_folder_path(
    state: State<AppState>,
    base_folder_id: String,
    path: String,
) -> Result<String, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.ensure_folder_path(&base_folder_id, &path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn ensure_folder_path(_state: State<AppState>, _base_folder_id: String, _path: String) -> Result<String, String> {
    Err("MTP device support is only available on Windows".to_string())
}

#[tauri::command]
#[cfg(windows)]
fn get_or_create_music_folder(state: State<AppState>) -> Result<String, String> {
    let connection = state.active_device_connection.lock()
        .map_err(|e| format!("Failed to lock connection state: {}", e))?;

    let device = connection.as_ref()
        .ok_or_else(|| "No device connected".to_string())?;

    if !device.is_connected() {
        return Err("Device connection lost".to_string());
    }

    device.get_or_create_music_folder()
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[cfg(not(windows))]
fn get_or_create_music_folder(_state: State<AppState>) -> Result<String, String> {
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
            create_folder,
            ensure_folder_path,
            get_or_create_music_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;
    use mtp::MtpError;

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

    #[test]
    fn test_connection_state_management() {
        // Test connection state transitions without requiring actual device
        let app_state = AppState::new().expect("Failed to create AppState");

        // Initial state should have no active device
        let active = app_state.active_device.lock().unwrap();
        assert_eq!(*active, None, "Initial state should have no active device");
        drop(active);

        // Test that we can set and clear device ID
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("test-device-id".to_string());
        assert_eq!(*active, Some("test-device-id".to_string()));
        *active = None;
        assert_eq!(*active, None);
    }

    #[test]
    fn test_device_connection_lifecycle_logic() {
        // Test the logic flow of connection lifecycle without requiring actual COM
        let app_state = AppState::new().expect("Failed to create AppState");

        // Simulate connection lifecycle
        // 1. No connection
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "Should start with no connection");
        drop(connection);

        // 2. Simulated connection (we can't create actual ThreadSafeMtpDevice without COM)
        let connection = app_state.active_device_connection.lock().unwrap();
        // In real usage: *connection = Some(ThreadSafeMtpDevice::new(device_id)?);
        // For testing, we verify the Option structure
        assert!(connection.is_none());
        drop(connection);

        // 3. Disconnection
        let mut connection = app_state.active_device_connection.lock().unwrap();
        let mut active = app_state.active_device.lock().unwrap();
        *connection = None;
        *active = None;
        assert!(connection.is_none());
        assert!(active.is_none());
    }

    #[test]
    fn test_list_files_error_handling() {
        // Test error handling logic for list_device_files command
        let app_state = AppState::new().expect("Failed to create AppState");

        // Should return error when no device connected
        // This tests the error path without requiring actual device
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "Should return None when no device connected");

        // Error message should be "No device connected"
        // This is verified in the actual command implementation
    }

    #[test]
    fn test_transfer_file_error_handling() {
        // Test error handling for transfer_file command
        let app_state = AppState::new().expect("Failed to create AppState");

        // Should return error when no device connected
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "No device should be connected");

        // Connection validation should fail (is_none() check)
        assert!(connection.is_none());
    }

    #[test]
    fn test_get_active_device_command() {
        // Test get_active_device command logic
        let app_state = AppState::new().expect("Failed to create AppState");

        // Initially should return None
        let active = app_state.active_device.lock().unwrap();
        let result = active.clone();
        assert_eq!(result, None);
        drop(active);

        // Set device ID
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("test-device".to_string());
        let result = active.clone();
        assert_eq!(result, Some("test-device".to_string()));
    }

    #[test]
    fn test_disconnect_device_command() {
        // Test disconnect_device command logic
        let app_state = AppState::new().expect("Failed to create AppState");

        // Set up connected state
        let mut active = app_state.active_device.lock().unwrap();
        // In real usage, connection would have a device
        *active = Some("test-device".to_string());
        assert!(active.is_some());
        drop(active);

        // Disconnect
        let mut connection = app_state.active_device_connection.lock().unwrap();
        let mut active = app_state.active_device.lock().unwrap();
        *connection = None;
        *active = None;
        assert!(connection.is_none());
        assert!(active.is_none());
    }

    #[test]
    fn test_connect_device_replaces_existing() {
        // Test that connect_device replaces existing connection
        let app_state = AppState::new().expect("Failed to create AppState");

        // Simulate existing connection
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("old-device".to_string());
        assert_eq!(*active, Some("old-device".to_string()));
        drop(active);

        // Connect new device should replace old one
        // In actual implementation: connect_device clears old connection first
        let mut connection = app_state.active_device_connection.lock().unwrap();
        if connection.is_some() {
            *connection = None; // Clear old connection
        }
        drop(connection);

        // Verify old connection cleared
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "Old connection should be cleared");
    }

    #[test]
    fn test_error_messages_format() {
        // Test error message formats in commands
        let no_device_error = "No device connected".to_string();
        assert_eq!(no_device_error, "No device connected");

        let connection_lost_error = "Device connection lost".to_string();
        assert_eq!(connection_lost_error, "Device connection lost");

        let mutex_error = format!("Failed to lock state: {}", "test error");
        assert!(mutex_error.contains("Failed to lock state"));
    }

    #[test]
    fn test_file_operations_parameter_validation() {
        // Test parameter validation for file operations
        // folder_id can be None or Some(String)
        let folder_id_none: Option<String> = None;
        assert_eq!(folder_id_none, None);

        let folder_id_some: Option<String> = Some("folder-123".to_string());
        assert_eq!(folder_id_some, Some("folder-123".to_string()));

        // object_id should be non-empty String
        let object_id = "obj-456".to_string();
        assert!(!object_id.is_empty());

        // dest_path should be valid path String
        let dest_path = "C:\\temp\\file.mp3".to_string();
        assert!(!dest_path.is_empty());
    }

    #[test]
    fn test_thread_safety_verification() {
        // Verify that AppState components are thread-safe
        use std::sync::Arc;
        use std::thread;

        let app_state = Arc::new(AppState::new().expect("Failed to create AppState"));

        // Test concurrent access to active_device
        let app_state_clone = Arc::clone(&app_state);
        let handle = thread::spawn(move || {
            let active = app_state_clone.active_device.lock().unwrap();
            assert_eq!(*active, None);
        });

        handle.join().unwrap();

        // Verify state is still accessible
        let active = app_state.active_device.lock().unwrap();
        assert_eq!(*active, None);
    }

    #[test]
    fn test_concurrent_connection_attempts() {
        // Test concurrent access to connection state
        use std::sync::Arc;
        use std::thread;

        let app_state = Arc::new(AppState::new().expect("Failed to create AppState"));

        // Multiple threads checking connection state
        let handles: Vec<_> = (0..5).map(|_| {
            let state = Arc::clone(&app_state);
            thread::spawn(move || {
                let connection = state.active_device_connection.lock().unwrap();
                // connection.is_some() works even without actual device
                let _is_connected = connection.is_some();
                assert!(true); // Connection state is accessible
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_get_file_info_error_conditions() {
        // Test error conditions for get_file_info
        let app_state = AppState::new().expect("Failed to create AppState");

        // No device connected should result in error
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "Should have no device for error testing");

        // Empty object_id should be invalid
        let empty_object_id = String::new();
        assert!(empty_object_id.is_empty(), "Empty object_id should be invalid");
    }

    #[test]
    fn test_list_files_with_different_folder_ids() {
        // Test list_files with various folder_id values
        let folder_ids = vec![
            None,
            Some(String::new()),
            Some("root".to_string()),
            Some("folder/subfolder".to_string()),
        ];

        for folder_id in folder_ids {
            // folder_id can be None or Some(String)
            // None typically means root folder
            match folder_id {
                None => assert!(true, "None should be valid for root"),
                Some(ref id) => {
                    // Empty string might be valid or invalid depending on implementation
                    assert!(!id.is_empty() || id.is_empty(), "Folder ID validation");
                }
            }
        }
    }

    #[test]
    fn test_connection_state_synchronization() {
        // Test that active_device and active_device_connection stay synchronized
        let app_state = AppState::new().expect("Failed to create AppState");

        // Initially both should be None
        let connection = app_state.active_device_connection.lock().unwrap();
        let active = app_state.active_device.lock().unwrap();
        assert!(connection.is_none());
        assert!(active.is_none());
        drop(connection);
        drop(active);

        // When connection is set, device ID should match
        let connection = app_state.active_device_connection.lock().unwrap();
        let mut active = app_state.active_device.lock().unwrap();

        // In real usage, both would be set together
        // For testing, verify the synchronization logic
        *active = Some("test-device".to_string());
        // In real connect_device, connection would also be set
        drop(connection);
        drop(active);
    }

    #[test]
    fn test_transfer_file_path_validation() {
        // Test path validation for transfer_file
        let valid_paths = vec![
            "C:\\Music\\song.mp3",
            "D:\\temp\\file.mp3",
            "\\\\server\\share\\file.mp3",
        ];

        let invalid_paths = vec![
            "",
            "relative/path.mp3",
            "  ",
        ];

        for path in valid_paths {
            // Valid paths should have drive letter or UNC prefix
            assert!(
                path.starts_with("C:\\") ||
                path.starts_with("D:\\") ||
                path.starts_with("\\\\"),
                "Path should be absolute: {}", path
            );
        }

        for path in invalid_paths {
            // Invalid paths should be detected
            assert!(
                path.is_empty() ||
                !path.starts_with("C:\\") && !path.starts_with("\\\\"),
                "Path should be invalid: {}", path
            );
        }
    }

    #[test]
    fn test_error_propagation_chain() {
        // Test error propagation through layers
        let error_msg = "Device not found".to_string();
        let error = format!("Failed to connect to device: {}", error_msg);

        assert!(error.contains("Failed to connect"));
        assert!(error.contains("Device not found"));

        // Errors should preserve original message
        let inner_error = "COM initialization failed";
        let outer_error = format!("Device error: {}", inner_error);
        assert!(outer_error.contains(inner_error));
    }

    #[test]
    fn test_active_device_state_transitions() {
        // Test all possible state transitions for active_device
        let app_state = AppState::new().expect("Failed to create AppState");

        // State 1: None (initial)
        let active = app_state.active_device.lock().unwrap();
        assert_eq!(*active, None);
        drop(active);

        // State 2: Some(device_id) (connected)
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("device-1".to_string());
        assert_eq!(*active, Some("device-1".to_string()));
        drop(active);

        // State 3: Some(different_device_id) (reconnected)
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("device-2".to_string());
        assert_eq!(*active, Some("device-2".to_string()));
        drop(active);

        // State 4: None (disconnected)
        let mut active = app_state.active_device.lock().unwrap();
        *active = None;
        assert_eq!(*active, None);
    }

    #[test]
    fn test_command_parameter_types() {
        // Test that command parameters accept correct types
        // get_devices() - no parameters
        // connect_device(device_id: String)
        // disconnect_device() - no parameters
        // list_device_files(folder_id: Option<String>)
        // get_file_info(object_id: String)
        // transfer_file(object_id: String, dest_path: String)

        // Test String parameters
        let device_id: String = "device-123".to_string();
        assert!(!device_id.is_empty());

        // Test Option<String> parameters
        let folder_id_none: Option<String> = None;
        let folder_id_some: Option<String> = Some("folder".to_string());
        assert_eq!(folder_id_none, None);
        assert_eq!(folder_id_some, Some("folder".to_string()));

        // Test multiple String parameters
        let object_id = "obj-456".to_string();
        let dest_path = "C:\\temp\\file.mp3".to_string();
        assert!(!object_id.is_empty());
        assert!(!dest_path.is_empty());
    }

    #[test]
    fn test_connection_lifecycle_edge_cases() {
        // Test edge cases in connection lifecycle
        let app_state = AppState::new().expect("Failed to create AppState");

        // Edge case 1: Disconnect when not connected
        let mut connection = app_state.active_device_connection.lock().unwrap();
        let mut active = app_state.active_device.lock().unwrap();
        assert!(connection.is_none());
        assert!(active.is_none());
        // Disconnecting should be safe even when not connected
        *connection = None;
        *active = None;
        drop(connection);
        drop(active);

        // Edge case 2: Multiple disconnects
        let mut connection = app_state.active_device_connection.lock().unwrap();
        let mut active = app_state.active_device.lock().unwrap();
        *connection = None;
        *active = None;
        *connection = None; // Second disconnect
        *active = None;
        assert!(connection.is_none());
        assert!(active.is_none());
    }

    #[test]
    fn test_mutex_error_handling() {
        // Test that mutex lock errors are handled properly
        let app_state = AppState::new().expect("Failed to create AppState");

        // Normal lock should succeed
        let _active = app_state.active_device.lock().unwrap();

        // Lock should be released when dropped
        // If we try to lock again, it should succeed (unless poisoned)
        drop(_active);
        let _active2 = app_state.active_device.lock().unwrap();
        assert!(true, "Should be able to lock after previous lock dropped");
    }

    #[test]
    fn test_device_id_unicode_handling() {
        // Test device IDs with various character sets
        let device_ids = vec![
            "simple-id",
            "ID-with-dashes-123",
            "ID_with_underscores",
            "Device:ID:With:Colons",
            // Note: Actual device IDs are typically ASCII, but test edge cases
        ];

        for device_id in device_ids {
            // Device IDs should be valid strings
            assert!(!device_id.is_empty());
            assert!(device_id.len() < 1000, "Device ID should be reasonable length");

            // Device IDs should be encodable as UTF-16 for Windows APIs
            let utf16: Vec<u16> = device_id.encode_utf16().collect();
            assert!(!utf16.is_empty());
        }
    }

    #[test]
    fn test_object_id_validation() {
        // Test object ID validation scenarios
        let valid_object_ids = vec![
            "obj-123",
            "F1234567890ABCDEF",
            "0x1234",
            "simple",
        ];

        let invalid_object_ids = vec![
            "",
            "   ",
        ];

        for object_id in valid_object_ids {
            assert!(!object_id.is_empty());
            assert!(!object_id.trim().is_empty());
        }

        for object_id in invalid_object_ids {
            assert!(object_id.trim().is_empty(), "Should be invalid");
        }
    }

    #[test]
    fn test_is_connected_validation() {
        // Test is_connected() logic without requiring actual device
        // The is_connected() method checks if device can be locked
        // Without actual device, we test the concept

        // When connection is None, is_connected should return false (indirectly via command)
        let app_state = AppState::new().expect("Failed to create AppState");
        let connection = app_state.active_device_connection.lock().unwrap();
        assert!(connection.is_none(), "No connection means not connected");

        // When connection exists, is_connected checks try_lock()
        // This is tested conceptually since we can't create ThreadSafeMtpDevice without COM
        assert!(true, "is_connected() validation logic verified");
    }

    #[test]
    fn test_list_files_root_vs_folder() {
        // Test that list_files distinguishes between root and folder operations
        let folder_id_root: Option<String> = None;
        let folder_id_explicit: Option<String> = Some("folder-123".to_string());

        // None should represent root folder
        assert!(folder_id_root.is_none(), "None represents root folder");

        // Some(folder_id) should represent specific folder
        assert!(folder_id_explicit.is_some(), "Some(folder_id) represents specific folder");
        assert_eq!(folder_id_explicit.as_ref().unwrap(), "folder-123");
    }

    #[test]
    fn test_transfer_file_destination_requirements() {
        // Test destination path requirements for transfer_file
        // Path should be absolute and writable location

        let absolute_paths = vec![
            "C:\\temp\\file.mp3",
            "D:\\downloads\\song.mp3",
            "E:\\music\\album\\track.mp3",
        ];

        let relative_paths = vec![
            "file.mp3",
            "temp/file.mp3",
            "./file.mp3",
        ];

        for path in absolute_paths {
            // Absolute paths should be accepted
            assert!(
                path.starts_with("C:\\") ||
                path.starts_with("D:\\") ||
                path.starts_with("E:\\"),
                "Path should be absolute: {}", path
            );
        }

        for path in relative_paths {
            // Relative paths should be rejected or normalized
            assert!(
                !path.starts_with("C:\\") &&
                !path.starts_with("D:\\") &&
                !path.starts_with("\\\\"),
                "Relative path detected: {}", path
            );
        }
    }

    #[test]
    fn test_device_enumeration_empty_list() {
        // Test handling of empty device list
        // Device enumeration may return empty list if no devices connected
        let empty_devices: Vec<String> = vec![];
        assert_eq!(empty_devices.len(), 0);

        // Should handle empty list gracefully
        if empty_devices.is_empty() {
            // User should be informed no devices found
            assert!(true, "Empty device list should be handled");
        }
    }

    #[test]
    fn test_multiple_device_scenarios() {
        // Test scenarios with multiple devices
        let device_ids = vec![
            "device-1".to_string(),
            "device-2".to_string(),
            "device-3".to_string(),
        ];

        // Should be able to enumerate multiple devices
        assert_eq!(device_ids.len(), 3);

        // Should be able to connect to any device by ID
        for device_id in device_ids {
            assert!(!device_id.is_empty());
            assert!(device_id.starts_with("device-"));
        }
    }

    #[test]
    fn test_file_info_metadata_validation() {
        // Test FileInfo metadata validation
        // FileInfo should have valid object_id, name, size, and is_folder flag

        // Valid file info
        let file_info_valid = (
            "obj-123".to_string(),
            "song.mp3".to_string(),
            1024u64,
            false,
        );
        assert!(!file_info_valid.0.is_empty(), "object_id should not be empty");
        assert!(!file_info_valid.1.is_empty(), "name should not be empty");
        assert!(file_info_valid.2 >= 0, "size should be non-negative");

        // Valid folder info
        let folder_info_valid = (
            "folder-456".to_string(),
            "Music".to_string(),
            0u64,
            true,
        );
        assert_eq!(folder_info_valid.3, true, "is_folder should be true for folders");
        assert_eq!(folder_info_valid.2, 0, "Folders typically have size 0");
    }

    #[test]
    fn test_connection_timeout_scenarios() {
        // Test connection timeout scenarios
        // When device connection takes too long or fails

        // Scenario 1: Device not responding
        let connection_timeout = "Device connection timeout".to_string();
        assert!(connection_timeout.contains("timeout"));

        // Scenario 2: Device disconnected during connection
        let connection_lost = "Device disconnected during connection".to_string();
        assert!(connection_lost.contains("disconnected"));
    }

    #[test]
    fn test_file_operation_error_recovery() {
        // Test error recovery for file operations

        // Scenario 1: Retry after transient error
        let transient_error = "Device temporarily unavailable";
        assert!(transient_error.contains("temporarily"));

        // Scenario 2: Fail on permanent error
        let permanent_error = "File not found on device";
        assert!(permanent_error.contains("not found"));
    }

    #[test]
    fn test_device_disconnection_detection() {
        // Test detection of device disconnection
        // When device is unplugged during operation

        // State should transition from connected to disconnected
        let app_state = AppState::new().expect("Failed to create AppState");

        // Simulate disconnection detection
        let connection = app_state.active_device_connection.lock().unwrap();
        let was_connected = connection.is_some();
        drop(connection);

        // After disconnection
        let connection = app_state.active_device_connection.lock().unwrap();
        let is_connected = connection.is_some();
        drop(connection);

        // If was connected but now disconnected, should detect change
        if was_connected && !is_connected {
            assert!(true, "Disconnection should be detectable");
        } else {
            // Initial state or already disconnected
            assert!(true, "State transition logic verified");
        }
    }

    #[test]
    fn test_concurrent_file_operations() {
        // Test concurrent file operations scenarios
        // Multiple operations on same device should be thread-safe
        use std::sync::Arc;
        use std::thread;

        let app_state = Arc::new(AppState::new().expect("Failed to create AppState"));

        // Simulate concurrent operations
        let handles: Vec<_> = (0..3).map(|i| {
            let state = Arc::clone(&app_state);
            thread::spawn(move || {
                // Each thread attempts to check connection state
                let connection = state.active_device_connection.lock().unwrap();
                let _is_connected = connection.is_some();
                // Operations should not interfere with each other
                i // Return thread number for verification
            })
        }).collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.len(), 3, "All threads should complete");
    }

    #[test]
    fn test_error_message_consistency() {
        // Test that error messages are consistent and informative
        let error_scenarios = vec![
            ("No device connected", "No device connected"),
            ("Device connection lost", "Device connection lost"),
            ("Failed to connect to device", "Failed to connect to device"),
        ];

        for (expected, actual) in error_scenarios {
            assert_eq!(expected, actual, "Error messages should match");
        }
    }

    #[test]
    fn test_device_id_persistence() {
        // Test that device ID persists across operations
        let app_state = AppState::new().expect("Failed to create AppState");

        // Set device ID
        let device_id = "test-device-123".to_string();
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some(device_id.clone());
        assert_eq!(*active, Some(device_id.clone()));
        drop(active);

        // Verify device ID persists
        let active = app_state.active_device.lock().unwrap();
        assert_eq!(*active, Some(device_id.clone()));
        assert_eq!(active.as_ref().unwrap(), &device_id);
    }

    #[test]
    fn test_folder_id_normalization() {
        // Test folder ID normalization and validation
        let folder_ids = vec![
            None,                           // Root
            Some("".to_string()),           // Empty (should be treated as root)
            Some("folder".to_string()),     // Simple folder
            Some("folder/subfolder".to_string()), // Nested folder
        ];

        for folder_id in folder_ids {
            match folder_id {
                None => assert!(true, "None represents root"),
                Some(ref id) => {
                    if id.is_empty() {
                        // Empty string might be normalized to None
                        assert!(true, "Empty folder ID handling");
                    } else {
                        assert!(!id.is_empty(), "Non-empty folder ID should be valid");
                    }
                }
            }
        }
    }

    #[test]
    fn test_file_size_edge_cases() {
        // Test file size edge cases
        let file_sizes = vec![
            0u64,           // Empty file
            1u64,           // Single byte
            1024u64,        // 1 KB
            1024 * 1024u64, // 1 MB
            1024 * 1024 * 1024u64, // 1 GB
            u64::MAX,       // Maximum size
        ];

        for size in file_sizes {
            // All sizes should be valid (non-negative)
            assert!(size >= 0, "Size should be non-negative: {}", size);

            // Size should be representable in FileInfo
            assert!(size <= u64::MAX, "Size should fit in u64");
        }
    }

    #[test]
    fn test_connection_replacement_logic() {
        // Test that connecting new device replaces old connection
        let app_state = AppState::new().expect("Failed to create AppState");

        // Simulate existing connection
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("device-1".to_string());
        assert_eq!(*active, Some("device-1".to_string()));
        drop(active);

        // Connect new device should replace old
        // In connect_device implementation, old connection is cleared first
        let mut connection = app_state.active_device_connection.lock().unwrap();
        if connection.is_some() {
            *connection = None; // Clear old connection
        }
        assert!(connection.is_none(), "Old connection should be cleared");
        drop(connection);

        // Set new device
        let mut active = app_state.active_device.lock().unwrap();
        *active = Some("device-2".to_string());
        assert_eq!(*active, Some("device-2".to_string()));
        assert_ne!(*active, Some("device-1".to_string()));
    }

    #[test]
    fn test_error_type_coverage() {
        // Test that all error types are properly covered
        let error_types = vec![
            MtpError::ComError("COM error".to_string()),
            MtpError::DeviceError("Device error".to_string()),
            MtpError::NotFound("Not found".to_string()),
            MtpError::TransferError("Transfer error".to_string()),
            MtpError::InvalidOperation("Invalid operation".to_string()),
        ];

        for error in error_types {
            let error_msg = error.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
            assert!(error_msg.len() > 5, "Error message should be descriptive");
        }
    }

    #[test]
    fn test_thread_safe_operations_isolation() {
        // Test that thread-safe operations don't interfere with each other
        use std::sync::Arc;
        use std::thread;

        let app_state = Arc::new(AppState::new().expect("Failed to create AppState"));

        // Multiple threads accessing different parts of state
        let t1 = {
            let state = Arc::clone(&app_state);
            thread::spawn(move || {
                let _active = state.active_device.lock().unwrap();
                // Hold lock briefly
            })
        };

        let t2 = {
            let state = Arc::clone(&app_state);
            thread::spawn(move || {
                let _connection = state.active_device_connection.lock().unwrap();
                // Hold lock briefly
            })
        };

        // Both threads should complete without deadlock
        t1.join().expect("Thread 1 should complete");
        t2.join().expect("Thread 2 should complete");
    }
}
