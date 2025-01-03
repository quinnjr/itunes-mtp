use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str,
};

use quick_xml::{events::{Event, BytesText}, Reader, name::QName};

use crate::mtp::MtpDevice;

use crate::errors::SyncError;

// A minimal Track struct
#[derive(Debug, Default)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub location: String,
}

// A minimal Playlist struct
#[derive(Debug, Default)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<String>, // Track IDs
}

// iTunesLibrary containing tracks and playlists
#[derive(Debug, Default)]
pub struct ITunesLibrary {
    pub tracks: HashMap<String, Track>,
    pub playlists: Vec<Playlist>,
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
        reader.trim_text(true);

        let mut library = ITunesLibrary::default();
        let mut current_track: Option<Track> = None;
        let mut current_playlist: Option<Playlist> = None;
        let mut in_tracks = false;
        let mut in_playlists = false;
        let mut current_key: Option<String> = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        name if name == QName(b"dict") && !in_tracks && !in_playlists => {
                            in_tracks = true;
                        }
                        name if name == QName(b"dict") && in_tracks && current_track.is_none() => {
                            current_track = Some(Track::default());
                        }
                        name if name == QName(b"array") && !in_tracks => {
                            in_playlists = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?;
                    let text_str = String::from_utf8(text.as_bytes().to_vec())?;
                    if let Some(track) = current_track.as_mut() {
                        if let Some(key) = &current_key {
                            match key.as_bytes() {
                                b"Name" => track.name = text_str,
                                b"Track ID" => track.id = text_str,
                                b"Artist" => track.artist = text_str,
                                b"Location" => track.location = text_str,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        name if name == QName(b"dict") && in_tracks && current_track.is_some() => {
                            let track = current_track.take().unwrap();
                            library.tracks.insert(track.id.clone(), track);
                        }
                        name if name == QName(b"dict") && in_tracks && current_track.is_none() => {
                            in_tracks = false;
                        }
                        name if name == QName(b"array") && in_playlists => {
                            in_playlists = false;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(SyncError::XmlError(e)),
                _ => {}
            }
        }
        Ok(library)
    }

    pub fn sync_to_mtp(&self, device: &MtpDevice) -> Result<(), SyncError> {
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