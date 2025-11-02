use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use quick_xml::{events::Event, Reader, name::QName};

#[cfg(windows)]
use crate::mtp::MtpDevice;

use crate::errors::SyncError;

// Enhanced Track struct with all metadata
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Option<u32>, // milliseconds
    pub play_count: Option<u32>,
    pub skip_count: Option<u32>,
    pub rating: Option<u32>,
    pub bpm: Option<u32>,
    pub date_added: Option<String>,
    pub date_modified: Option<String>,
    pub last_played: Option<String>,
    pub skip_date: Option<String>,
    pub comments: Option<String>,
    pub kind: Option<String>, // File type (e.g., "MPEG audio file")
    pub size: Option<u64>, // File size in bytes
    pub location: String,
}

// Enhanced Playlist struct
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<String>, // Track IDs
    pub is_folder: bool,
}

// iTunesLibrary containing tracks and playlists
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ITunesLibrary {
    pub tracks: HashMap<String, Track>,
    pub playlists: Vec<Playlist>,
}

/// Decode iTunes file:// URL to a proper file path
/// iTunes stores file paths as `file://localhost/C:/Music/...` URLs
/// This function converts them to Windows paths like `C:\Music\...`
///
/// Handles:
/// - `file://localhost/C:/Music/...` → `C:\Music\...`
/// - `file:///C:/Music/...` → `C:\Music\...`
/// - Network paths: `file://server/share/path` → `\\server\share\path`
/// - URL encoding: `%20` → space, `%28` → `(`, etc.
/// - Special characters in paths
fn decode_itunes_url(url: &str) -> String {
    // If not a file:// URL, return as-is
    if !url.starts_with("file://") {
        return url.to_string();
    }

    // Remove the file:// prefix
    let mut path = url.strip_prefix("file://").unwrap_or(url).to_string();

    // Remove leading slashes (can be file:/// or file://localhost/)
    while path.starts_with('/') {
        path.remove(0);
    }

    // Check if this is localhost (local file path)
    let is_localhost = path.starts_with("localhost/");
    if is_localhost {
        // Local file path with localhost
        path = path.strip_prefix("localhost/").unwrap_or(&path).to_string();
    }

    // Check if this is a network path (file://server/share/...)
    // Network paths: don't have localhost, don't have drive letter (C:/), have at least one /
    let is_network_path = !is_localhost &&
                          !path.chars().next().map(|c| c.is_ascii_alphabetic() && path.chars().nth(1) == Some(':')).unwrap_or(false) &&
                          path.contains('/');

    if is_network_path {
        // Handle network paths (file://server/share/path)
        let decoded = urlencoding::decode(&path).unwrap_or(std::borrow::Cow::Borrowed(path.as_str()));
        #[cfg(target_os = "windows")]
        {
            let result = decoded.replace('/', "\\");
            // Ensure network path format: \\server\share\path
            if !result.starts_with("\\\\") {
                format!("\\\\{}", result)
            } else {
                result
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            decoded.to_string()
        }
    } else {
        // Local file path
        // Decode URL encoding (e.g., %20 -> space, %28 -> (, %29 -> ))
        let decoded = urlencoding::decode(&path).unwrap_or(std::borrow::Cow::Borrowed(path.as_str()));

        // Convert forward slashes to backslashes on Windows
        #[cfg(target_os = "windows")]
        {
            decoded.replace('/', "\\").to_string()
        }
        #[cfg(not(target_os = "windows"))]
        {
            decoded.to_string()
        }
    }
}

// Application state
#[derive(Default)]
pub struct AppState {
    pub library_path: Option<PathBuf>,
    pub library: Option<ITunesLibrary>,
}

impl AppState {
    pub fn parse_library(&mut self, path: &Path) -> Result<ITunesLibrary, SyncError> {
        let xml = fs::read_to_string(path)?;
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut library = ITunesLibrary::default();

        // Parse state
        let mut in_tracks_dict = false;
        let mut in_playlists_array = false;
        let mut in_playlist_items = false;
        let mut current_track: Option<Track> = None;
        let mut current_playlist: Option<Playlist> = None;
        let mut current_key: Option<String> = None;
        let mut next_is_key = false;
        let mut dict_depth = 0;
        let mut tracks_dict_depth = 0;
        let mut playlist_dict_depth = 0;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    match e.name() {
                        QName(b"key") => {
                            next_is_key = true;
                        }
                        QName(b"dict") => {
                            dict_depth += 1;

                            // Check if this is the tracks dictionary
                            if !in_tracks_dict && current_key.as_deref() == Some("Tracks") {
                                in_tracks_dict = true;
                                tracks_dict_depth = dict_depth;
                                current_key = None; // Clear the key after using it
                            }
                            // Check if we're starting a new track (track dict is one level deeper than tracks dict)
                            else if in_tracks_dict && dict_depth == tracks_dict_depth + 1 {
                                current_track = Some(Track::default());
                            }
                            // Check if we're starting a new playlist (playlist dict is one level deeper than array)
                            else if in_playlists_array && dict_depth == playlist_dict_depth + 1 {
                                if current_playlist.is_none() {
                                    current_playlist = Some(Playlist::default());
                                }
                            }
                        }
                        QName(b"array") => {
                            dict_depth += 1;
                            // Check if this is the playlists array
                            if current_key.as_deref() == Some("Playlists") {
                                in_playlists_array = true;
                                playlist_dict_depth = dict_depth;
                                current_key = None; // Clear the key after using it
                            }
                            // Check if this is the playlist items array
                            else if in_playlists_array && current_key.as_deref() == Some("Playlist Items") {
                                in_playlist_items = true;
                                current_key = None; // Clear the key after using it
                            }
                        }
                        QName(b"string") | QName(b"integer") | QName(b"date") => {
                            // Will be handled in Text event
                        }
                        QName(b"true") => {
                            if let Some(playlist) = current_playlist.as_mut() {
                                if current_key.as_deref() == Some("Folder") {
                                    playlist.is_folder = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?;
                    let text_str = text.trim().to_string();

                    if next_is_key {
                        current_key = Some(text_str);
                        next_is_key = false;
                    } else if let Some(key) = &current_key {
                        // Parse track fields
                        if let Some(track) = current_track.as_mut() {
                            match key.as_str() {
                                "Track ID" => track.id = text_str,
                                "Name" => track.name = text_str,
                                "Artist" => track.artist = text_str,
                                "Album" => track.album = Some(text_str),
                                "Album Artist" => track.album_artist = Some(text_str),
                                "Genre" => track.genre = Some(text_str),
                                "Composer" => track.composer = Some(text_str),
                                "Year" => track.year = text_str.parse().ok(),
                                "Track Number" => track.track_number = text_str.parse().ok(),
                                "Disc Number" => track.disc_number = text_str.parse().ok(),
                                "Total Time" => track.duration = text_str.parse().ok(),
                                "Play Count" => track.play_count = text_str.parse().ok(),
                                "Skip Count" => track.skip_count = text_str.parse().ok(),
                                "Rating" => track.rating = text_str.parse().ok(),
                                "BPM" => track.bpm = text_str.parse().ok(),
                                "Date Added" => track.date_added = Some(text_str.clone()),
                                "Date Modified" => track.date_modified = Some(text_str.clone()),
                                "Play Date UTC" | "Play Date" => track.last_played = Some(text_str.clone()),
                                "Skip Date" => track.skip_date = Some(text_str.clone()),
                                "Comments" => track.comments = Some(text_str),
                                "Kind" => track.kind = Some(text_str),
                                "Size" => track.size = text_str.parse().ok(),
                                "Location" => track.location = decode_itunes_url(&text_str),
                                _ => {}
                            }
                        }
                        // Parse playlist fields
                        else if let Some(playlist) = current_playlist.as_mut() {
                            match key.as_str() {
                                "Name" => playlist.name = text_str,
                                "Playlist ID" => playlist.id = text_str,
                                "Playlist Persistent ID" => {
                                    if playlist.id.is_empty() {
                                        playlist.id = text_str;
                                    }
                                }
                                "Track ID" => {
                                    if in_playlist_items {
                                        playlist.tracks.push(text_str);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        QName(b"dict") => {
                            dict_depth -= 1;

                            // End of a track (track dict is at tracks_dict_depth + 1)
                            if in_tracks_dict && dict_depth == tracks_dict_depth {
                                if let Some(track) = current_track.take() {
                                    if !track.id.is_empty() {
                                        library.tracks.insert(track.id.clone(), track);
                                    }
                                }
                            }
                            // End of tracks dictionary
                            else if in_tracks_dict && dict_depth == tracks_dict_depth - 1 {
                                in_tracks_dict = false;
                            }
                            // End of a playlist (after decrement, playlist dict was at playlist_dict_depth + 1)
                            else if in_playlists_array && dict_depth == playlist_dict_depth {
                                if let Some(playlist) = current_playlist.take() {
                                    // Only add playlists that aren't the master playlist
                                    if !playlist.name.is_empty() && playlist.name != "Library" {
                                        library.playlists.push(playlist);
                                    }
                                }
                            }
                        }
                        QName(b"array") => {
                            dict_depth -= 1;
                            if in_playlist_items {
                                in_playlist_items = false;
                            } else if in_playlists_array && dict_depth == playlist_dict_depth {
                                // End of playlists array
                                in_playlists_array = false;
                            }
                        }
                        QName(b"key") => {
                            next_is_key = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(SyncError::XmlError(e)),
                _ => {}
            }
        }

        self.library_path = Some(path.to_path_buf());
        self.library = Some(library.clone());
        Ok(library)
    }

    #[cfg(windows)]
    pub fn sync_to_mtp(&self, _device: &MtpDevice) -> Result<(), SyncError> {
        unimplemented!();
    }

    pub fn generate_mtp_playlist_content(
        &self,
        playlist: &Playlist,
    ) -> Result<String, SyncError> {
        let mut content = String::from("#EXTM3U\n");

        if let Some(library) = &self.library {
            for track_id in &playlist.tracks {
                if let Some(track) = library.tracks.get(track_id) {
                    let mtp_path = format!("/Music/{}/{}/{}",
                        track.artist,
                        track.name,
                        Path::new(&track.location)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .ok_or_else(|| SyncError::ParseError("Invalid filename in playlist".to_string()))?
                    );

                    content.push_str(&format!("#EXTINF:-1,{} - {}\n", track.artist, track.name));
                    content.push_str(&mtp_path);
                    content.push('\n');
                }
            }
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const SAMPLE_ITUNES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Major Version</key><integer>1</integer>
    <key>Minor Version</key><integer>1</integer>
    <key>Application Version</key><string>12.0.0.0</string>
    <key>Features</key><integer>5</integer>
    <key>Show Content Ratings</key><true/>
    <key>Music Folder</key><string>file://localhost/C:/Music/</string>
    <key>Library Persistent ID</key><string>1234567890ABCDEF</string>
    <key>Tracks</key>
    <dict>
        <key>1</key>
        <dict>
            <key>Track ID</key><integer>1</integer>
            <key>Name</key><string>Test Song 1</string>
            <key>Artist</key><string>Test Artist</string>
            <key>Album</key><string>Test Album</string>
            <key>Album Artist</key><string>Test Album Artist</string>
            <key>Genre</key><string>Rock</string>
            <key>Composer</key><string>Test Composer</string>
            <key>Year</key><integer>2020</integer>
            <key>Track Number</key><integer>1</integer>
            <key>Disc Number</key><integer>1</integer>
            <key>Total Time</key><integer>180000</integer>
            <key>Play Count</key><integer>5</integer>
            <key>Skip Count</key><integer>1</integer>
            <key>Rating</key><integer>80</integer>
            <key>BPM</key><integer>120</integer>
            <key>Date Added</key><date>2020-01-01T12:00:00Z</date>
            <key>Date Modified</key><date>2020-01-02T12:00:00Z</date>
            <key>Play Date UTC</key><date>2020-01-03T12:00:00Z</date>
            <key>Skip Date</key><date>2020-01-04T12:00:00Z</date>
            <key>Comments</key><string>Test comment</string>
            <key>Kind</key><string>MPEG audio file</string>
            <key>Size</key><integer>3456789</integer>
            <key>Location</key><string>file://localhost/C:/Music/Test%20Artist/Test%20Album/Test%20Song%201.mp3</string>
        </dict>
        <key>2</key>
        <dict>
            <key>Track ID</key><integer>2</integer>
            <key>Name</key><string>Test Song 2</string>
            <key>Artist</key><string>Another Artist</string>
            <key>Album</key><string>Another Album</string>
            <key>Genre</key><string>Pop</string>
            <key>Year</key><integer>2021</integer>
            <key>Total Time</key><integer>200000</integer>
            <key>Date Added</key><date>2021-01-01T12:00:00Z</date>
            <key>Location</key><string>file://localhost/C:/Music/Another%20Artist/Another%20Album/Test%20Song%202.mp3</string>
        </dict>
    </dict>
    <key>Playlists</key>
    <array>
        <dict>
            <key>Name</key><string>Library</string>
            <key>Master</key><true/>
            <key>Playlist ID</key><integer>100</integer>
            <key>Playlist Persistent ID</key><string>MASTER001</string>
            <key>All Items</key><true/>
        </dict>
        <dict>
            <key>Name</key><string>My Favorites</string>
            <key>Playlist ID</key><integer>101</integer>
            <key>Playlist Persistent ID</key><string>FAV001</string>
            <key>All Items</key><false/>
            <key>Playlist Items</key>
            <array>
                <dict>
                    <key>Track ID</key><integer>1</integer>
                </dict>
                <dict>
                    <key>Track ID</key><integer>2</integer>
                </dict>
            </array>
        </dict>
        <dict>
            <key>Name</key><string>Rock Music</string>
            <key>Playlist ID</key><integer>102</integer>
            <key>Playlist Persistent ID</key><string>ROCK001</string>
            <key>All Items</key><false/>
            <key>Playlist Items</key>
            <array>
                <dict>
                    <key>Track ID</key><integer>1</integer>
                </dict>
            </array>
        </dict>
        <dict>
            <key>Name</key><string>My Folder</string>
            <key>Playlist ID</key><integer>103</integer>
            <key>Playlist Persistent ID</key><string>FOLDER001</string>
            <key>Folder</key><true/>
        </dict>
    </array>
</dict>
</plist>
"#;

    #[test]
    fn test_parse_tracks() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(SAMPLE_ITUNES_XML.as_bytes()).expect("Failed to write temp file");

        let mut state = AppState::default();
        let library = state.parse_library(temp_file.path()).expect("Failed to parse library");

        // Check we parsed 2 tracks
        assert_eq!(library.tracks.len(), 2, "Should parse 2 tracks");

        // Check track 1 - verify all metadata fields
        let track1 = library.tracks.get("1").expect("Track 1 should exist");
        assert_eq!(track1.id, "1");
        assert_eq!(track1.name, "Test Song 1");
        assert_eq!(track1.artist, "Test Artist");
        assert_eq!(track1.album, Some("Test Album".to_string()));
        assert_eq!(track1.album_artist, Some("Test Album Artist".to_string()));
        assert_eq!(track1.genre, Some("Rock".to_string()));
        assert_eq!(track1.composer, Some("Test Composer".to_string()));
        assert_eq!(track1.year, Some(2020));
        assert_eq!(track1.track_number, Some(1));
        assert_eq!(track1.disc_number, Some(1));
        assert_eq!(track1.duration, Some(180000));
        assert_eq!(track1.play_count, Some(5));
        assert_eq!(track1.skip_count, Some(1));
        assert_eq!(track1.rating, Some(80));
        assert_eq!(track1.bpm, Some(120));
        assert!(track1.date_added.is_some());
        assert!(track1.date_modified.is_some());
        assert!(track1.last_played.is_some());
        assert!(track1.skip_date.is_some());
        assert_eq!(track1.comments, Some("Test comment".to_string()));
        assert_eq!(track1.kind, Some("MPEG audio file".to_string()));
        assert_eq!(track1.size, Some(3456789));

        // Check track 2
        let track2 = library.tracks.get("2").expect("Track 2 should exist");
        assert_eq!(track2.id, "2");
        assert_eq!(track2.name, "Test Song 2");
        assert_eq!(track2.artist, "Another Artist");
        assert_eq!(track2.album, Some("Another Album".to_string()));
    }

    #[test]
    fn test_parse_playlists() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(SAMPLE_ITUNES_XML.as_bytes()).expect("Failed to write temp file");

        let mut state = AppState::default();
        let library = state.parse_library(temp_file.path()).expect("Failed to parse library");

        // Should parse 3 playlists (excluding Library master playlist)
        assert_eq!(library.playlists.len(), 3, "Should parse 3 playlists");

        // Check "My Favorites" playlist
        let favorites = library.playlists.iter()
            .find(|p| p.name == "My Favorites")
            .expect("Favorites playlist should exist");
        assert_eq!(favorites.id, "101");
        assert_eq!(favorites.tracks.len(), 2, "Favorites should have 2 tracks");
        assert_eq!(favorites.tracks[0], "1");
        assert_eq!(favorites.tracks[1], "2");
        assert!(!favorites.is_folder);

        // Check "Rock Music" playlist
        let rock = library.playlists.iter()
            .find(|p| p.name == "Rock Music")
            .expect("Rock playlist should exist");
        assert_eq!(rock.id, "102");
        assert_eq!(rock.tracks.len(), 1, "Rock should have 1 track");
        assert_eq!(rock.tracks[0], "1");

        // Check folder
        let folder = library.playlists.iter()
            .find(|p| p.name == "My Folder")
            .expect("Folder should exist");
        assert_eq!(folder.id, "103");
        assert!(folder.is_folder, "Should be marked as folder");
        assert_eq!(folder.tracks.len(), 0, "Folder should have no tracks");
    }

    #[test]
    fn test_decode_itunes_url() {
        // Test file:// URL with localhost
        assert_eq!(
            decode_itunes_url("file://localhost/C:/Music/Test%20Song.mp3"),
            if cfg!(target_os = "windows") {
                "C:\\Music\\Test Song.mp3"
            } else {
                "C:/Music/Test Song.mp3"
            }
        );

        // Test file:// URL without localhost (three slashes)
        assert_eq!(
            decode_itunes_url("file:///C:/Music/Song.mp3"),
            if cfg!(target_os = "windows") {
                "C:\\Music\\Song.mp3"
            } else {
                "C:/Music/Song.mp3"
            }
        );

        // Test URL with special characters (parentheses, spaces)
        assert_eq!(
            decode_itunes_url("file://localhost/C:/Music/Artist%20Name/Album%20%28Deluxe%29/01%20Track.mp3"),
            if cfg!(target_os = "windows") {
                "C:\\Music\\Artist Name\\Album (Deluxe)\\01 Track.mp3"
            } else {
                "C:/Music/Artist Name/Album (Deluxe)/01 Track.mp3"
            }
        );

        // Test URL with various special characters
        assert_eq!(
            decode_itunes_url("file://localhost/C:/Music/Song%20%26%20Dance%20%23%201.mp3"),
            if cfg!(target_os = "windows") {
                "C:\\Music\\Song & Dance # 1.mp3"
            } else {
                "C:/Music/Song & Dance # 1.mp3"
            }
        );

        // Test network path (UNC path)
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                decode_itunes_url("file://server/share/music/song.mp3"),
                "\\\\server\\share\\music\\song.mp3"
            );

            assert_eq!(
                decode_itunes_url("file://SERVER-01/Music%20Library/Artist/Album/Song.mp3"),
                "\\\\SERVER-01\\Music Library\\Artist\\Album\\Song.mp3"
            );
        }

        // Test non-URL path (should return as-is)
        assert_eq!(
            decode_itunes_url("C:\\Music\\Song.mp3"),
            "C:\\Music\\Song.mp3"
        );

        // Test relative path (should return as-is)
        assert_eq!(
            decode_itunes_url("Music/Song.mp3"),
            "Music/Song.mp3"
        );

        // Test empty string
        assert_eq!(
            decode_itunes_url(""),
            ""
        );
    }

    #[test]
    fn test_generate_mtp_playlist_content() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(SAMPLE_ITUNES_XML.as_bytes()).expect("Failed to write temp file");

        let mut state = AppState::default();
        let library = state.parse_library(temp_file.path()).expect("Failed to parse library");

        let favorites = library.playlists.iter()
            .find(|p| p.name == "My Favorites")
            .expect("Favorites playlist should exist");

        let content = state.generate_mtp_playlist_content(favorites)
            .expect("Failed to generate playlist content");

        assert!(content.starts_with("#EXTM3U\n"), "Should start with M3U header");
        assert!(content.contains("Test Artist - Test Song 1"), "Should contain track 1 info");
        assert!(content.contains("Another Artist - Test Song 2"), "Should contain track 2 info");
        assert!(content.contains("/Music/Test Artist/Test Song 1/"), "Should contain track 1 path");
    }

    #[test]
    fn test_empty_library() {
        const EMPTY_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Tracks</key>
    <dict>
    </dict>
    <key>Playlists</key>
    <array>
    </array>
</dict>
</plist>
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(EMPTY_XML.as_bytes()).expect("Failed to write temp file");

        let mut state = AppState::default();
        let library = state.parse_library(temp_file.path()).expect("Failed to parse library");

        assert_eq!(library.tracks.len(), 0, "Should have no tracks");
        assert_eq!(library.playlists.len(), 0, "Should have no playlists");
    }

    #[test]
    fn test_track_without_optional_fields() {
        const MINIMAL_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Tracks</key>
    <dict>
        <key>1</key>
        <dict>
            <key>Track ID</key><integer>1</integer>
            <key>Name</key><string>Minimal Track</string>
            <key>Artist</key><string>Unknown</string>
            <key>Location</key><string>file://localhost/C:/Music/track.mp3</string>
        </dict>
    </dict>
    <key>Playlists</key>
    <array>
    </array>
</dict>
</plist>
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(MINIMAL_XML.as_bytes()).expect("Failed to write temp file");

        let mut state = AppState::default();
        let library = state.parse_library(temp_file.path()).expect("Failed to parse library");

        assert_eq!(library.tracks.len(), 1, "Should parse 1 track");

        let track = library.tracks.get("1").expect("Track should exist");
        assert_eq!(track.name, "Minimal Track");
        assert_eq!(track.artist, "Unknown");
        assert_eq!(track.album, None, "Album should be None");
        assert_eq!(track.genre, None, "Genre should be None");
        assert_eq!(track.year, None, "Year should be None");
        assert_eq!(track.duration, None, "Duration should be None");
    }
}
