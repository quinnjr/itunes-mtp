# Project Roadmap - iTunes to MTP Sync

This roadmap outlines the planned development phases for completing the iTunes library to MTP device synchronization application.

## 📍 Current Status

**Phase 0: Foundation** ✅ **COMPLETE**
- ✅ Basic project structure (Angular + Tauri)
- ✅ MTP device enumeration and connection
- ✅ Basic iTunes XML parsing (tracks only)
- ✅ UI framework and routing
- ✅ Service architecture with signals

---

## 🎯 Phase 1: Core iTunes Library Parsing (MVP Foundation)

**Goal:** Complete iTunes library parsing to extract all necessary data for sync

**Estimated Time:** 2-3 weeks

### Milestones
1. **Complete Playlist Parsing**
   - Fix XML parser to extract playlists from `<array>` elements
   - Parse playlist names, track IDs, and relationships
   - Handle nested playlists (playlist folders)
   - Store playlist hierarchy

2. **Enhanced Track Metadata**
   - Parse additional track fields (Album, Genre, Year, Duration)
   - Store extended metadata in Track struct
   - Update TypeScript models to match

3. **File Path Resolution**
   - Decode iTunes `file://` URLs to Windows file paths
   - Handle `file://localhost/` and network paths
   - Validate file existence
   - Handle special characters and encoding

4. **Library Validation**
   - Check all track files exist before sync
   - Report missing files to user
   - Calculate total size and track counts
   - Generate library statistics

**Dependencies:** None - Foundation work

**Success Criteria:**
- Can parse complete iTunes library XML files
- All playlists and tracks extracted correctly
- File paths resolved and validated
- Library stats displayed in UI

---

## 🔌 Phase 2: MTP Device Write Operations

**Goal:** Implement file upload and folder creation on MTP devices

**Estimated Time:** 3-4 weeks

### Milestones
1. **Persistent Device Connection** ✅ **COMPLETE**
   - ✅ Maintain device connection in AppState - IMPLEMENTED
   - ✅ Reuse IPortableDevice instance - IMPLEMENTED
   - ✅ Handle connection lifecycle properly - IMPLEMENTED
   - ✅ Implement connection pooling/reuse - IMPLEMENTED
   - ✅ Added ThreadSafeMtpDevice wrapper for thread-safe connection reuse
   - ✅ Connection validation and graceful disconnection handling

2. **Folder Management**
   - Create folders on device (`/Music`, `/Music/Artist`, etc.)
   - Check if folders exist before creating
   - Navigate folder hierarchy
   - Support configurable folder structures

3. **File Upload Implementation**
   - Implement `upload_file_to_device()` function
   - Use IPortableDeviceDataStream for writing
   - Stream files in chunks (avoid memory issues)
   - Set file metadata during upload (name, size, date)

4. **Storage Management** ✅ **COMPLETE**
   - ✅ Query available storage on device - IMPLEMENTED
   - ✅ StorageInfo struct and get_storage_info() method added
   - ✅ Tauri command and TypeScript service method implemented
   - ✅ Used space calculated from file enumeration
   - [ ] Calculate required space before sync (to be implemented in sync logic)
   - [ ] Warn user if insufficient space (to be implemented in UI)
   - [ ] Display storage usage in UI (to be implemented)
   - Note: Most MTP devices don't expose standard storage capacity properties
     through the Windows Portable Device API. The current implementation
     calculates used space by summing file sizes. Total and free space
     querying would require device-specific implementations or proprietary APIs.

**Dependencies:** Phase 1 (need valid file paths to upload)

**Success Criteria:**
- Can create folder structure on device
- Can upload files to device successfully
- Storage checks prevent sync if space insufficient
- Files appear correctly on device after upload

---

## 🔄 Phase 3: Core Sync Functionality

**Goal:** Implement end-to-end playlist synchronization

**Estimated Time:** 4-5 weeks

### Milestones
1. **Sync Implementation**
   - Complete `sync_playlist_to_device()` function
   - For each track: resolve path → upload → update progress
   - Handle errors per-track (continue on failure)
   - Create M3U playlist files on device

2. **Progress Tracking**
   - Real-time progress updates per file
   - Progress per playlist
   - Overall sync progress percentage
   - Use Tauri events for progress updates
   - Speed and ETA calculation

3. **Error Handling**
   - Retry logic for failed transfers (with backoff)
   - Skip corrupted files with warnings
   - Continue sync after individual failures
   - Generate detailed error report after sync

4. **Duplicate Detection**
   - Check if files already exist on device
   - Compare by filename, size, or checksum
   - User options: skip, overwrite, or rename
   - Maintain sync manifest/metadata

**Dependencies:** Phase 1, Phase 2

**Success Criteria:**
- Can sync selected playlists to device end-to-end
- Progress displayed accurately
- Errors handled gracefully
- Sync completes successfully with error reporting

---

## 🎨 Phase 4: User Experience Enhancements

**Goal:** Polish UI/UX and add user-friendly features

**Estimated Time:** 2-3 weeks

### Milestones
1. **Sync Progress UI**
   - Real-time progress bar with percentage
   - Current file/playlist display
   - Speed and ETA indicators
   - Cancel button functionality
   - Detailed status messages

2. **Playlist Selection Improvements**
   - Search and filter playlists
   - Show track count and size per playlist
   - Playlist preview/details
   - Bulk select/deselect operations
   - Visual selection indicators

3. **Device Management UI**
   - Better device list with icons/info
   - Connection status indicators
   - Device storage usage display
   - Connection/disconnection controls

4. **Error Display & Handling**
   - User-friendly error messages
   - Post-sync error summary
   - Detailed sync report
   - Retry failed items option

**Dependencies:** Phase 3 (need working sync to show progress)

**Success Criteria:**
- Intuitive and polished user interface
- Clear feedback during all operations
- Easy to understand error messages
- Professional appearance and UX

---

## 🧪 Phase 5: Testing & Quality Assurance

**Goal:** Comprehensive test coverage and bug fixes

**Estimated Time:** 3-4 weeks

### Milestones
1. **Unit Tests**
   - iTunes XML parser tests (80%+ coverage)
   - MTP operations tests (with mocks)
   - Sync service logic tests
   - Error handling tests

2. **Integration Tests**
   - End-to-end sync workflow
   - Device interaction tests
   - File transfer tests
   - Error recovery tests

3. **UI Component Tests**
   - Component rendering tests
   - User interaction tests
   - Service integration tests

4. **Performance & Stress Testing**
   - Large library handling (1000+ tracks)
   - Many playlists (100+ playlists)
   - Large files (high-res audio)
   - Concurrent operations

5. **Bug Fixes**
   - Address issues found in testing
   - Performance optimizations
   - Memory leak fixes
   - Edge case handling

**Dependencies:** Phases 1-4 (need features to test)

**Success Criteria:**
- 80%+ test coverage across codebase
- All critical paths tested
- Performance acceptable for large libraries
- No critical bugs remaining

---

## 🚀 Phase 6: Advanced Features

**Goal:** Add value-added features for power users

**Estimated Time:** 4-6 weeks (optional)

### Milestones
1. **Sync Options & Configuration**
   - Settings/preferences screen
   - Configurable folder structures
   - File format conversion options
   - Sync behavior preferences

2. **Sync Resume & Recovery**
   - Save sync state/progress
   - Resume interrupted syncs
   - Skip already-transferred files
   - Sync manifest/database

3. **Sync Preview & Dry-Run**
   - Preview what will be transferred
   - Show file sizes and estimated time
   - Review before actual transfer
   - Export preview report

4. **Format Conversion**
   - Convert to device-compatible formats
   - Re-encode audio files (bitrate, format)
   - Resize album artwork
   - Quality settings

**Dependencies:** Phase 5 (stable foundation)

**Success Criteria:**
- Advanced features working reliably
- Improved sync experience
- Better device compatibility

---

## 📚 Phase 7: Documentation & Polish

**Goal:** Complete documentation and final polish

**Estimated Time:** 2 weeks

### Milestones
1. **User Documentation**
   - Installation guide
   - How to export iTunes library XML
   - Step-by-step sync guide
   - Troubleshooting guide
   - FAQ

2. **Developer Documentation**
   - Architecture overview
   - Code structure guide
   - Contributing guidelines
   - Testing guide

3. **Final Polish**
   - UI/UX refinements
   - Error message improvements
   - Performance optimizations
   - Code cleanup and refactoring

**Dependencies:** All previous phases

**Success Criteria:**
- Complete user documentation
- Code well-documented
- Application polished and ready for release

---

## 📊 Release Timeline

### MVP Release (Phases 1-4)
**Target:** 12-15 weeks from start
- Complete iTunes library parsing
- MTP device write operations
- Core sync functionality
- Basic user interface

### Stable Release (Phases 1-5)
**Target:** 15-19 weeks from start
- MVP + comprehensive testing
- Bug fixes and quality assurance
- Production-ready application

### Feature Complete (Phases 1-7)
**Target:** 21-27 weeks from start
- All planned features
- Advanced options
- Complete documentation

---

## 🔄 Ongoing Priorities

These should be considered throughout all phases:

1. **Code Quality**
   - Follow Rust and TypeScript best practices
   - Maintain 80%+ test coverage
   - Code reviews and refactoring

2. **Performance**
   - Optimize file transfers
   - Minimize memory usage
   - Efficient XML parsing

3. **Error Handling**
   - Graceful error recovery
   - User-friendly messages
   - Comprehensive logging

4. **Security**
   - Safe file operations
   - Input validation
   - Path sanitization

---

## 📝 Notes

- **Flexibility:** This roadmap is flexible and may be adjusted based on:
  - User feedback during development
  - Technical discoveries
  - Priority changes
  - Resource availability

- **Parallel Work:** Some phases can be worked on in parallel:
  - UI improvements (Phase 4) can start while backend (Phase 3) is being tested
  - Documentation (Phase 7) can begin during testing phase

- **MVP Focus:** Phases 1-4 represent the minimum viable product. Phases 5-7 add quality and polish.

- **Future Considerations:**
  - Two-way sync (device → iTunes)
  - Cloud storage integration
  - Multi-device sync
  - Smart playlist evaluation
  - Metadata editing

