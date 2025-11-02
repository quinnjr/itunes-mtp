# TODO - iTunes to MTP Sync Project

This file tracks remaining tasks to complete the iTunes library to MTP device synchronization functionality.

## 🎵 iTunes Library Parsing

### High Priority
- [ ] **Complete playlist parsing in Rust XML parser**
  - Currently only tracks are parsed, playlists are not being extracted from XML
  - Need to parse `<array>` elements containing playlist structures
  - Parse playlist names, track IDs, and playlist hierarchy (smart playlists, folders)

- [ ] **Parse all track metadata**
  - Currently parsing: Name, Artist, Location, Track ID
  - Missing: Album, Genre, Year, Duration, Play Count, Rating, Date Added
  - Store in Track struct for better organization

- [ ] **Handle iTunes file:// URL decoding**
  - iTunes stores file paths as `file://localhost/...` URLs
  - Need to decode and convert to Windows file paths (e.g., `file://localhost/C:/Music/...` → `C:\Music\...`)
  - Handle special characters and path encoding

- [ ] **Validate parsed library data**
  - Check that track files exist on disk before sync
  - Report missing files to user
  - Handle relative vs absolute paths

### Medium Priority
- [ ] **Parse nested playlists (playlist folders)**
  - Support for playlist folders/containers
  - Handle "smart playlists" (may need evaluation or export)

- [ ] **Track deduplication**
  - Detect when same file appears in multiple playlists
  - Only transfer once, reference in all playlists

- [ ] **Library statistics**
  - Total size calculation
  - Estimated sync time
  - Storage space validation

## 📱 MTP Device Management

### High Priority
- [x] **Implement persistent device connection** ✅
  - ✅ Currently reconnecting on each operation (inefficient) - FIXED
  - ✅ Maintain device connection state in AppState - IMPLEMENTED
  - ✅ Reuse IPortableDevice instance across operations - IMPLEMENTED
  - ✅ Added ThreadSafeMtpDevice wrapper for connection reuse
  - ✅ Connection lifecycle management implemented
  - ✅ Graceful disconnection handling

- [ ] **Create folder structure on device**
  - Create `/Music` folder if it doesn't exist
  - Create artist folders (`/Music/Artist Name`)
  - Create album folders (`/Music/Artist Name/Album Name`)
  - Support user-configurable folder structure

- [ ] **Upload files to device (write operation)**
  - Currently only `transfer_file` reads FROM device
  - Need to implement `upload_file` or `copy_file_to_device`
  - Use IPortableDeviceDataStream or IStream for writing
  - Handle file metadata during upload

- [x] **Check available storage space** ✅
  - ✅ Query device storage capacity - IMPLEMENTED (calculates used space)
  - ✅ StorageInfo struct with total_space, free_space, used_space fields
  - ✅ Tauri command get_device_storage_info() exposed
  - ✅ TypeScript service method getStorageInfo() added
  - [ ] Verify enough space before starting sync (to be implemented in sync logic)
  - [ ] Warn user if insufficient space (to be implemented in UI)
  - Note: Most MTP devices don't expose standard storage capacity through WPD API.
    Current implementation calculates used space from file enumeration.
    Total/free space may require device-specific implementations.

### Medium Priority
- [ ] **Device file browsing enhancements**
  - Show file types/icons
  - Display file metadata (size, date modified)
  - Support file deletion (for cleanup/re-sync)

- [ ] **Device information display**
  - Show device storage usage
  - Display supported formats
  - Show connection status

- [ ] **Handle device disconnection gracefully**
  - Detect when device is unplugged during sync
  - Resume sync on reconnect (optional)
  - Show appropriate error messages

## 🔄 Sync Functionality

### High Priority
- [ ] **Implement complete sync_playlist_to_device**
  - Currently just a stub that returns success message
  - For each track in playlist:
    - Resolve file path from iTunes location
    - Verify file exists
    - Upload file to device (creating folders as needed)
    - Update progress
  - Create M3U playlist file on device
  - Handle errors per track (continue with others)

- [ ] **File transfer progress tracking**
  - Report progress per file (bytes transferred)
  - Report progress per playlist
  - Report overall sync progress
  - Use Tauri events to emit progress updates

- [ ] **Duplicate detection**
  - Check if file already exists on device (by name/size/checksum)
  - Option to skip, overwrite, or rename
  - Maintain sync manifest/database

- [ ] **Error handling and retry logic**
  - Retry failed transfers (with backoff)
  - Skip corrupted files with warning
  - Continue sync after individual failures
  - Generate detailed error report

### Medium Priority
- [ ] **Sync resume capability**
  - Save sync state/progress
  - Allow resuming interrupted syncs
  - Skip already-transferred files

- [ ] **Sync preview/dry-run mode**
  - Show what would be transferred
  - Show file sizes and estimated time
  - Allow user to review before actual transfer

- [ ] **Sync options**
  - Choose folder structure (Artist/Album, Flat, etc.)
  - Option to convert file formats
  - Option to re-encode for device compatibility
  - Sync metadata (artwork, ratings)

- [ ] **Batch operations**
  - Transfer multiple playlists concurrently
  - Optimize transfer order
  - Queue management

## 🎨 User Interface

### High Priority
- [ ] **Sync progress UI**
  - Real-time progress bar with percentage
  - Current file being transferred
  - Speed/ETA display
  - Cancel button
  - Detailed status messages

- [ ] **Playlist selection improvements**
  - Search/filter playlists
  - Show track count per playlist
  - Show playlist size
  - Select/deselect all with checkboxes
  - Playlist preview

- [ ] **Device connection UI**
  - Better device list display
  - Device status indicators
  - Connection/disconnection controls
  - Device storage info

- [ ] **Error display**
  - User-friendly error messages
  - Error summary after sync
  - Logs/console output option
  - Retry failed items

### Medium Priority
- [ ] **Settings/preferences screen**
  - Default sync folder
  - Folder structure preference
  - File format options
  - Sync behavior settings

- [ ] **Library information display**
  - Detailed library stats
  - Track list view
  - Search functionality
  - Filter by artist/genre

- [ ] **Sync history**
  - Show previous syncs
  - Last sync date/time
  - Sync statistics

## 🧪 Testing

### High Priority
- [ ] **Unit tests for iTunes XML parser**
  - Test with sample XML files
  - Test track parsing
  - Test playlist parsing (once implemented)
  - Test edge cases (missing fields, malformed XML)

- [ ] **Unit tests for MTP operations**
  - Mock MTP device for testing
  - Test device enumeration
  - Test file operations (read, write)
  - Test error conditions

- [ ] **Unit tests for sync service**
  - Test sync logic
  - Test progress tracking
  - Test error handling
  - Test duplicate detection

- [ ] **Integration tests**
  - End-to-end sync test (with mock device)
  - Test full workflow (upload library → select playlists → sync)

### Medium Priority
- [ ] **UI component tests**
  - Test file upload
  - Test playlist selection
  - Test sync progress display

- [ ] **Performance tests**
  - Large library handling
  - Many playlists
  - Large files

## 🐛 Bug Fixes & Improvements

### High Priority
- [ ] **Fix iTunes XML playlist parsing**
  - `current_playlist` variable is declared but never used
  - `current_key` is set but never read properly
  - Playlist parsing logic is incomplete

- [ ] **Fix file path handling**
  - Proper URL decoding for iTunes file paths
  - Handle network paths
  - Handle special characters in paths

- [ ] **Memory management**
  - Avoid loading entire library into memory
  - Stream large files during transfer
  - Proper cleanup of COM objects

### Medium Priority
- [ ] **Performance optimizations**
  - Cache device connection
  - Batch file operations
  - Parallel transfers where possible

- [ ] **Code refactoring**
  - Better error types
  - Consistent error handling
  - Better separation of concerns

## 📚 Documentation

- [ ] **User documentation**
  - How to export iTunes library XML
  - How to connect MTP device
  - How to sync playlists
  - Troubleshooting guide

- [ ] **Developer documentation**
  - Architecture overview
  - Adding new features
  - Testing guide
  - Code style guide

## 🚀 Future Enhancements

- [ ] **Two-way sync**
  - Sync device playlists back to iTunes
  - Merge playlists

- [ ] **Format conversion**
  - Convert to device-compatible formats
  - Re-encode audio files
  - Resize artwork

- [ ] **Advanced playlist features**
  - Sync smart playlists
  - Create device-specific playlists
  - Playlist size limits

- [ ] **Cloud integration**
  - Sync to cloud storage
  - Backup playlists

- [ ] **Multi-device support**
  - Sync to multiple devices
  - Device profiles

