export interface Track {
  id: string;
  name: string;
  artist: string;
  location: string;
}

export interface Playlist {
  name: string;
  tracks: string[]; // Track IDs
  trackCount?: number; // Optional for display purposes
}

export interface ITunesLibrary {
  tracks: Record<string, Track>;
  playlists: Playlist[];
}

export interface LibraryState {
  isLoaded: boolean;
  fileName: string;
  trackCount: number;
  playlistCount: number;
  library: ITunesLibrary | null;
  error: string | null;
}
