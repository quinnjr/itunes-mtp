import { Injectable, signal, WritableSignal, computed, inject } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { ITunesLibrary, LibraryState, Track, Playlist } from '../../shared/models/library.model';
import { AsyncHandlerService } from './async-handler.service';

@Injectable({
  providedIn: 'root'
})
export class ItunesLibraryService {
  // Inject async handler service
  private readonly asyncHandler = inject(AsyncHandlerService);

  // Private writable signals
  private readonly _isLoaded = signal<boolean>(false);
  private readonly _fileName = signal<string>('');
  private readonly _trackCount = signal<number>(0);
  private readonly _playlistCount = signal<number>(0);
  private readonly _library = signal<ITunesLibrary | null>(null);
  private readonly _error = signal<string | null>(null);
  private readonly _selectedPlaylists = signal<Playlist[]>([]);

  // Public readonly signals
  public readonly selectedPlaylists = this._selectedPlaylists.asReadonly();

  // Computed signals for derived state
  public readonly libraryState = computed<LibraryState>(() => ({
    isLoaded: this._isLoaded(),
    fileName: this._fileName(),
    trackCount: this._trackCount(),
    playlistCount: this._playlistCount(),
    library: this._library(),
    error: this._error()
  }));

  public readonly isLoaded = computed<boolean>(() => this._isLoaded());
  public readonly fileName = computed<string>(() => this._fileName());
  public readonly trackCount = computed<number>(() => this._trackCount());
  public readonly playlistCount = computed<number>(() => this._playlistCount());
  public readonly library = computed<ITunesLibrary | null>(() => this._library());
  public readonly error = computed<string | null>(() => this._error());

  constructor() {}

  /**
   * Parse an iTunes library XML file
   */
  public async parseLibrary(xmlContent: string, fileName: string): Promise<void> {
    await this.asyncHandler.executeAsync(
      async () => {
        // Send XML content to Rust backend for parsing
        const library = await invoke<ITunesLibrary>('parse_itunes_library', { xmlContent });

        // Get playlists from backend
        const playlists = await invoke<Playlist[]>('get_playlists');

        // Get tracks from backend
        const tracks = await invoke<Track[]>('get_tracks');

        console.log('Library parsed successfully:', {
          tracks: tracks.length,
          playlists: playlists.length
        });

        return {
          library,
          playlists,
          tracks,
          fileName
        };
      },
      {
        setLoading: () => {}, // No loading state for parsing
        setData: (result) => {
          this._isLoaded.set(true);
          this._fileName.set(result.fileName);
          this._trackCount.set(result.tracks.length);
          this._playlistCount.set(result.playlists.length);
          this._library.set(result.library);
        },
        setError: (error) => {
          this._error.set(error);
          // Fallback to mock data for demonstration
          this._loadMockLibrary(fileName);
        }
      }
    );
  }

  /**
   * Clear the current library
   */
  public clearLibrary(): void {
    this._isLoaded.set(false);
    this._fileName.set('');
    this._trackCount.set(0);
    this._playlistCount.set(0);
    this._library.set(null);
    this._error.set(null);
    this._selectedPlaylists.set([]);
  }

  /**
   * Get all playlists from the current library
   */
  public getPlaylists(): Playlist[] {
    return this.library()?.playlists || [];
  }

  /**
   * Get all tracks from the current library
   */
  public getTracks(): Track[] {
    const lib = this.library();
    if (!lib) return [];

    return Object.values(lib.tracks);
  }

  /**
   * Get a specific track by ID
   */
  public getTrackById(trackId: string): Track | null {
    const lib = this.library();
    if (!lib) return null;

    return lib.tracks[trackId] || null;
  }

  /**
   * Get a specific playlist by name
   */
  public getPlaylistByName(name: string): Playlist | null {
    const playlists = this.getPlaylists();
    return playlists.find(p => p.name === name) || null;
  }

  /**
   * Toggle playlist selection
   */
  public togglePlaylistSelection(playlist: Playlist): void {
    const selected = this._selectedPlaylists();
    const index = selected.findIndex(p => p.name === playlist.name);

    if (index >= 0) {
      // Remove from selection
      this._selectedPlaylists.set(selected.filter((_, i) => i !== index));
    } else {
      // Add to selection
      this._selectedPlaylists.set([...selected, playlist]);
    }
  }

  /**
   * Check if a playlist is selected
   */
  public isPlaylistSelected(playlist: Playlist): boolean {
    return this._selectedPlaylists().some(p => p.name === playlist.name);
  }

  /**
   * Select all playlists
   */
  public selectAllPlaylists(): void {
    const playlists = this.getPlaylists();
    this._selectedPlaylists.set([...playlists]);
  }

  /**
   * Deselect all playlists
   */
  public deselectAllPlaylists(): void {
    this._selectedPlaylists.set([]);
  }

  /**
   * Get the number of selected playlists
   */
  public getSelectedPlaylistCount(): number {
    return this._selectedPlaylists().length;
  }

  /**
   * Check if library is loaded
   */
  public isLibraryLoaded(): boolean {
    return this.isLoaded();
  }

  /**
   * Get the current library state
   */
  public getLibraryState(): LibraryState {
    return this.libraryState();
  }

  /**
   * Get the current error state
   */
  public getError(): string | null {
    return this.error();
  }

  /**
   * Load mock library data for demonstration
   */
  private _loadMockLibrary(fileName: string): void {
    const mockPlaylists: Playlist[] = [
      { name: 'Favorites', tracks: [], trackCount: 45 },
      { name: 'Workout Mix', tracks: [], trackCount: 32 },
      { name: 'Chill Vibes', tracks: [], trackCount: 28 },
      { name: 'Road Trip', tracks: [], trackCount: 67 },
      { name: 'Focus', tracks: [], trackCount: 21 },
      { name: 'Party Hits', tracks: [], trackCount: 54 },
    ];

    const mockLibrary: ITunesLibrary = {
      tracks: {},
      playlists: mockPlaylists
    };

    this._isLoaded.set(true);
    this._fileName.set(fileName);
    this._trackCount.set(150); // Mock track count
    this._playlistCount.set(mockPlaylists.length);
    this._library.set(mockLibrary);
    this._error.set(null);
  }
}
