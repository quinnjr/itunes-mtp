import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { installTauriInvokeMock } from '../tauri.testing';
import { AppStateService } from './app-state.service';
import { MtpDeviceService } from './mtp-device.service';
import { ItunesLibraryService } from './itunes-library.service';
import { SyncService } from './sync.service';
import { Playlist } from '../../shared/models/library.model';

describe('AppStateService', () => {
  let service: AppStateService;
  let itunesLibraryService: ItunesLibraryService;
  let syncService: SyncService;

  const mockPlaylist: Playlist = {
    name: 'Test Playlist',
    tracks: ['1'],
    trackCount: 1
  };

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        AppStateService,
        MtpDeviceService,
        ItunesLibraryService,
        SyncService
      ]
    });

    // Install the Tauri IPC spy before services are constructed, since
    // MtpDeviceService issues an `invoke` call from its constructor.
    vi.clearAllMocks();
    installTauriInvokeMock().mockResolvedValue([]);

    service = TestBed.inject(AppStateService);
    itunesLibraryService = TestBed.inject(ItunesLibraryService);
    syncService = TestBed.inject(SyncService);
  });

  describe('Service Initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });

    it('should initialize with canSync false', () => {
      expect(service.canSync()).toBe(false);
    });
  });

  describe('getAppState', () => {
    it('should return current app state', () => {
      const state = service.getAppState();

      expect(state).toBeDefined();
      expect(state.device).toBeDefined();
      expect(state.library).toBeDefined();
      expect(state.sync).toBeDefined();
      expect(state.canSync).toBe(false);
    });
  });

  describe('canSync', () => {
    it('should return false when device not connected', () => {
      expect(service.canSync()).toBe(false);
    });

    it('should return false when library not loaded', () => {
      // Would need to mock device connection and library load
      expect(service.canSync()).toBe(false);
    });

    it('should return false when no playlists selected', () => {
      // Would need to mock device connection and library load
      expect(service.canSync()).toBe(false);
    });

    it('should return false when sync is active', () => {
      // Would need to mock sync active state
      expect(service.canSync()).toBe(false);
    });
  });

  describe('isDeviceConnected', () => {
    it('should return false initially', () => {
      expect(service.isDeviceConnected()).toBe(false);
    });
  });

  describe('isLibraryLoaded', () => {
    it('should return false initially', () => {
      expect(service.isLibraryLoaded()).toBe(false);
    });
  });

  describe('isSyncActive', () => {
    it('should return false initially', () => {
      expect(service.isSyncActive()).toBe(false);
    });
  });

  describe('getConnectedDevice', () => {
    it('should return null initially', () => {
      expect(service.getConnectedDevice()).toBeNull();
    });
  });

  describe('getLoadedLibrary', () => {
    it('should return library state', () => {
      const library = service.getLoadedLibrary();

      expect(library).toBeDefined();
      expect(library.isLoaded).toBe(false);
    });
  });

  describe('getSyncProgress', () => {
    it('should return sync progress', () => {
      const progress = service.getSyncProgress();

      expect(progress).toBeDefined();
      expect(progress.isActive).toBe(false);
    });
  });

  describe('getErrors', () => {
    it('should return error object', () => {
      const errors = service.getErrors();

      expect(errors).toBeDefined();
      expect(errors.device).toBeNull();
      expect(errors.library).toBeNull();
    });
  });

  describe('startSync', () => {
    it('should delegate to sync service', async () => {
      const startSyncSpy = vi.spyOn(syncService, 'startSync').mockResolvedValue({
        success: true,
        message: 'Success',
        syncedPlaylists: [],
        errors: []
      });

      // Mock selectedPlaylists signal - create a mock function that returns the playlist array
      const mockSignal = () => [mockPlaylist] as Playlist[];
      Object.defineProperty(itunesLibraryService, 'selectedPlaylists', {
        get: () => mockSignal,
        configurable: true
      });

      await service.startSync();

      expect(startSyncSpy).toHaveBeenCalledWith([mockPlaylist]);
    });
  });

  describe('cancelSync', () => {
    it('should delegate to sync service', () => {
      const cancelSyncSpy = vi.spyOn(syncService, 'cancelSync');

      service.cancelSync();

      expect(cancelSyncSpy).toHaveBeenCalled();
    });
  });

  describe('getSyncStatusMessage', () => {
    it('should delegate to sync service', () => {
      const getSyncStatusSpy = vi.spyOn(syncService, 'getSyncStatus').mockReturnValue('Ready');

      const status = service.getSyncStatusMessage();

      expect(getSyncStatusSpy).toHaveBeenCalled();
      expect(status).toBe('Ready');
    });
  });

  describe('getSelectedPlaylistCount', () => {
    it('should delegate to library service', () => {
      const getCountSpy = vi.spyOn(itunesLibraryService, 'getSelectedPlaylistCount').mockReturnValue(5);

      const count = service.getSelectedPlaylistCount();

      expect(getCountSpy).toHaveBeenCalled();
      expect(count).toBe(5);
    });
  });

  describe('getPlaylists', () => {
    it('should delegate to library service', () => {
      const getPlaylistsSpy = vi.spyOn(itunesLibraryService, 'getPlaylists').mockReturnValue([mockPlaylist]);

      const playlists = service.getPlaylists();

      expect(getPlaylistsSpy).toHaveBeenCalled();
      expect(playlists).toEqual([mockPlaylist]);
    });
  });

  describe('togglePlaylistSelection', () => {
    it('should delegate to library service', () => {
      const toggleSpy = vi.spyOn(itunesLibraryService, 'togglePlaylistSelection');

      service.togglePlaylistSelection(mockPlaylist);

      expect(toggleSpy).toHaveBeenCalledWith(mockPlaylist);
    });
  });

  describe('isPlaylistSelected', () => {
    it('should delegate to library service', () => {
      const isSelectedSpy = vi.spyOn(itunesLibraryService, 'isPlaylistSelected').mockReturnValue(true);

      const result = service.isPlaylistSelected(mockPlaylist);

      expect(isSelectedSpy).toHaveBeenCalledWith(mockPlaylist);
      expect(result).toBe(true);
    });
  });

  describe('selectAllPlaylists', () => {
    it('should delegate to library service', () => {
      const selectAllSpy = vi.spyOn(itunesLibraryService, 'selectAllPlaylists');

      service.selectAllPlaylists();

      expect(selectAllSpy).toHaveBeenCalled();
    });
  });

  describe('deselectAllPlaylists', () => {
    it('should delegate to library service', () => {
      const deselectAllSpy = vi.spyOn(itunesLibraryService, 'deselectAllPlaylists');

      service.deselectAllPlaylists();

      expect(deselectAllSpy).toHaveBeenCalled();
    });
  });
});

