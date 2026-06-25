import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { installTauriInvokeMock } from '../tauri.testing';
import { ItunesLibraryService } from './itunes-library.service';
import { AsyncHandlerService } from './async-handler.service';
import { Track, Playlist, ITunesLibrary } from '../../shared/models/library.model';

describe('ItunesLibraryService', () => {
  let service: ItunesLibraryService;
  // Tauri IPC spy used by the service; (re)installed in beforeEach.
  let invoke: Mock;

  const mockTrack: Track = {
    id: '1',
    name: 'Test Song',
    artist: 'Test Artist',
    location: 'file://localhost/C:/Music/test.mp3'
  };

  const mockPlaylist: Playlist = {
    name: 'Test Playlist',
    tracks: ['1'],
    trackCount: 1
  };

  const mockLibrary: ITunesLibrary = {
    tracks: { '1': mockTrack },
    playlists: [mockPlaylist]
  };

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [ItunesLibraryService, AsyncHandlerService]
    });

    // Install the Tauri IPC spy before the service is constructed.
    vi.clearAllMocks();
    invoke = installTauriInvokeMock();

    service = TestBed.inject(ItunesLibraryService);
  });

  describe('Service Initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });

    it('should initialize with library not loaded', () => {
      expect(service.isLoaded()).toBe(false);
    });

    it('should initialize with empty fileName', () => {
      expect(service.fileName()).toBe('');
    });

    it('should initialize with zero track count', () => {
      expect(service.trackCount()).toBe(0);
    });

    it('should initialize with zero playlist count', () => {
      expect(service.playlistCount()).toBe(0);
    });
  });

  describe('parseLibrary', () => {
    it('should parse library successfully', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary) // parse_itunes_library
        .mockResolvedValueOnce([mockPlaylist]) // get_playlists
        .mockResolvedValueOnce([mockTrack]); // get_tracks

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(invoke).toHaveBeenCalledWith('parse_itunes_library', {
        xmlContent: '<xml>content</xml>'
      });
      expect(invoke).toHaveBeenCalledWith('get_playlists');
      expect(invoke).toHaveBeenCalledWith('get_tracks');

      expect(service.isLoaded()).toBe(true);
      expect(service.fileName()).toBe('test.xml');
      expect(service.trackCount()).toBe(1);
      expect(service.playlistCount()).toBe(1);
    });

    it('should load mock library on error', async () => {
      invoke.mockRejectedValue(new Error('Parse failed'));

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(service.isLoaded()).toBe(true);
      expect(service.fileName()).toBe('test.xml');
      expect(service.playlistCount()).toBeGreaterThan(0);
    });
  });

  describe('clearLibrary', () => {
    beforeEach(async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);
      await service.parseLibrary('<xml>content</xml>', 'test.xml');
    });

    it('should clear library state', () => {
      service.clearLibrary();

      expect(service.isLoaded()).toBe(false);
      expect(service.fileName()).toBe('');
      expect(service.trackCount()).toBe(0);
      expect(service.playlistCount()).toBe(0);
      expect(service.library()).toBeNull();
      expect(service.error()).toBeNull();
      expect(service.selectedPlaylists().length).toBe(0);
    });
  });

  describe('getPlaylists', () => {
    it('should return empty array when no library loaded', () => {
      expect(service.getPlaylists()).toEqual([]);
    });

    it('should return playlists when library loaded', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      const playlists = service.getPlaylists();
      expect(playlists.length).toBe(1);
      expect(playlists[0]).toEqual(mockPlaylist);
    });
  });

  describe('getTracks', () => {
    it('should return empty array when no library loaded', () => {
      expect(service.getTracks()).toEqual([]);
    });

    it('should return tracks when library loaded', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      const tracks = service.getTracks();
      expect(tracks.length).toBe(1);
      expect(tracks[0]).toEqual(mockTrack);
    });
  });

  describe('getTrackById', () => {
    it('should return null when no library loaded', () => {
      expect(service.getTrackById('1')).toBeNull();
    });

    it('should return track when found', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(service.getTrackById('1')).toEqual(mockTrack);
    });

    it('should return null when track not found', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(service.getTrackById('999')).toBeNull();
    });
  });

  describe('getPlaylistByName', () => {
    it('should return null when playlist not found', () => {
      expect(service.getPlaylistByName('NonExistent')).toBeNull();
    });

    it('should return playlist when found', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(service.getPlaylistByName('Test Playlist')).toEqual(mockPlaylist);
    });
  });

  describe('togglePlaylistSelection', () => {
    beforeEach(async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);
      await service.parseLibrary('<xml>content</xml>', 'test.xml');
    });

    it('should add playlist to selection', () => {
      expect(service.isPlaylistSelected(mockPlaylist)).toBe(false);

      service.togglePlaylistSelection(mockPlaylist);

      expect(service.isPlaylistSelected(mockPlaylist)).toBe(true);
      expect(service.selectedPlaylists().length).toBe(1);
    });

    it('should remove playlist from selection', () => {
      service.togglePlaylistSelection(mockPlaylist);
      expect(service.isPlaylistSelected(mockPlaylist)).toBe(true);

      service.togglePlaylistSelection(mockPlaylist);

      expect(service.isPlaylistSelected(mockPlaylist)).toBe(false);
      expect(service.selectedPlaylists().length).toBe(0);
    });
  });

  describe('selectAllPlaylists', () => {
    beforeEach(async () => {
      const multiplePlaylists: Playlist[] = [
        mockPlaylist,
        { name: 'Playlist 2', tracks: [], trackCount: 0 },
        { name: 'Playlist 3', tracks: [], trackCount: 0 }
      ];
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce(multiplePlaylists)
        .mockResolvedValueOnce([mockTrack]);
      await service.parseLibrary('<xml>content</xml>', 'test.xml');
    });

    it('should select all playlists', () => {
      service.selectAllPlaylists();

      expect(service.selectedPlaylists().length).toBeGreaterThan(0);
    });
  });

  describe('deselectAllPlaylists', () => {
    beforeEach(async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);
      await service.parseLibrary('<xml>content</xml>', 'test.xml');
      service.togglePlaylistSelection(mockPlaylist);
    });

    it('should deselect all playlists', () => {
      expect(service.selectedPlaylists().length).toBe(1);

      service.deselectAllPlaylists();

      expect(service.selectedPlaylists().length).toBe(0);
    });
  });

  describe('getSelectedPlaylistCount', () => {
    it('should return zero when no playlists selected', () => {
      expect(service.getSelectedPlaylistCount()).toBe(0);
    });

    it('should return count of selected playlists', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);
      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      service.togglePlaylistSelection(mockPlaylist);

      expect(service.getSelectedPlaylistCount()).toBe(1);
    });
  });

  describe('isLibraryLoaded', () => {
    it('should return false when not loaded', () => {
      expect(service.isLibraryLoaded()).toBe(false);
    });

    it('should return true when loaded', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      expect(service.isLibraryLoaded()).toBe(true);
    });
  });

  describe('getLibraryState', () => {
    it('should return library state', async () => {
      invoke
        .mockResolvedValueOnce(mockLibrary)
        .mockResolvedValueOnce([mockPlaylist])
        .mockResolvedValueOnce([mockTrack]);

      await service.parseLibrary('<xml>content</xml>', 'test.xml');

      const state = service.getLibraryState();
      expect(state.isLoaded).toBe(true);
      expect(state.fileName).toBe('test.xml');
      expect(state.trackCount).toBe(1);
      expect(state.playlistCount).toBe(1);
    });
  });

  describe('getError', () => {
    it('should return null when no error', () => {
      expect(service.getError()).toBeNull();
    });
  });
});

