import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { SyncService } from './sync.service';
import { Playlist } from '../../shared/models/library.model';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('SyncService', () => {
  let service: SyncService;

  const mockPlaylist1: Playlist = {
    name: 'Playlist 1',
    tracks: ['1', '2'],
    trackCount: 2
  };

  const mockPlaylist2: Playlist = {
    name: 'Playlist 2',
    tracks: ['3'],
    trackCount: 1
  };

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [SyncService]
    });

    service = TestBed.inject(SyncService);
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Service Initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });

    it('should initialize with sync not active', () => {
      expect(service.isActive()).toBe(false);
    });

    it('should initialize with zero percentage', () => {
      expect(service.percentage()).toBe(0);
    });

    it('should initialize with no sync result', () => {
      expect(service.getLastSyncResult()).toBeNull();
    });
  });

  describe('startSync', () => {
    it('should return error when no playlists selected', async () => {
      const result = await service.startSync([]);

      expect(result.success).toBe(false);
      expect(result.message).toContain('No playlists selected');
      expect(result.syncedPlaylists.length).toBe(0);
      expect(result.errors.length).toBe(1);
    });

    it('should sync single playlist successfully', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');

      const result = await service.startSync([mockPlaylist1], 'Music');

      expect(invoke).toHaveBeenCalledWith('sync_playlist_to_device', {
        playlistName: 'Playlist 1',
        deviceFolder: 'Music'
      });
      expect(result.success).toBe(true);
      expect(result.syncedPlaylists).toContain('Playlist 1');
      expect(result.errors.length).toBe(0);
    });

    it('should sync multiple playlists successfully', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');

      const result = await service.startSync([mockPlaylist1, mockPlaylist2], 'Music');

      expect(invoke).toHaveBeenCalledTimes(2);
      expect(result.success).toBe(true);
      expect(result.syncedPlaylists.length).toBe(2);
    });

    it('should handle errors during sync', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Sync failed'));

      const result = await service.startSync([mockPlaylist1], 'Music');

      expect(result.success).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.syncedPlaylists.length).toBe(0);
    });

    it('should handle partial failures', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('Sync completed') // First playlist succeeds
        .mockRejectedValueOnce(new Error('Sync failed')); // Second playlist fails

      const result = await service.startSync([mockPlaylist1, mockPlaylist2], 'Music');

      expect(result.success).toBe(false);
      expect(result.syncedPlaylists.length).toBe(1);
      expect(result.errors.length).toBe(1);
    });

    it('should update progress during sync', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');

      // Fast-forward timers to complete delays
      await vi.runAllTimersAsync();

      await syncPromise;

      expect(service.syncProgress().completedPlaylists).toBe(2);
      expect(service.syncProgress().totalPlaylists).toBe(2);
      expect(service.syncProgress().percentage).toBe(100);
    });

    it('should use default folder when not specified', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');

      await service.startSync([mockPlaylist1]);

      expect(invoke).toHaveBeenCalledWith('sync_playlist_to_device', {
        playlistName: 'Playlist 1',
        deviceFolder: 'Music'
      });
    });
  });

  describe('cancelSync', () => {
    it('should cancel active sync', () => {
      // Start a sync (don't await to keep it active)
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {
        // Never resolves - intentionally empty
      }));
      service.startSync([mockPlaylist1], 'Music');

      service.cancelSync();

      expect(service.isActive()).toBe(false);
      expect(service.getSyncStatus()).toContain('cancelled');
    });
  });

  describe('isSyncActive', () => {
    it('should return false when not active', () => {
      expect(service.isSyncActive()).toBe(false);
    });

    it('should return true when sync is active', async () => {
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {
        // Never resolves - intentionally empty
      }));
      service.startSync([mockPlaylist1], 'Music');

      expect(service.isSyncActive()).toBe(true);
    });
  });

  describe('getSyncProgress', () => {
    it('should return progress with zero values initially', () => {
      const progress = service.getSyncProgress();
      expect(progress.isActive).toBe(false);
      expect(progress.completedPlaylists).toBe(0);
      expect(progress.totalPlaylists).toBe(0);
      expect(progress.percentage).toBe(0);
    });

    it('should return progress during sync', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');
      const syncPromise = service.startSync([mockPlaylist1], 'Music');

      await vi.runAllTimersAsync();
      await syncPromise;

      const progress = service.getSyncProgress();
      expect(progress.totalPlaylists).toBe(1);
      expect(progress.completedPlaylists).toBe(1);
      expect(progress.percentage).toBe(100);
    });
  });

  describe('getLastSyncResult', () => {
    it('should return null initially', () => {
      expect(service.getLastSyncResult()).toBeNull();
    });

    it('should return last sync result', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');
      await vi.runAllTimersAsync();
      const result = await service.startSync([mockPlaylist1], 'Music');

      expect(service.getLastSyncResult()).toEqual(result);
    });
  });

  describe('clearSyncResult', () => {
    it('should clear sync result', async () => {
      vi.mocked(invoke).mockResolvedValue('Sync completed');
      await vi.runAllTimersAsync();
      await service.startSync([mockPlaylist1], 'Music');

      service.clearSyncResult();

      expect(service.getLastSyncResult()).toBeNull();
    });
  });

  describe('getSyncStatus', () => {
    it('should return ready message when not active', () => {
      expect(service.getSyncStatus()).toBe('Ready to sync');
    });

    it('should return status with playlist name during sync', async () => {
      vi.mocked(invoke).mockImplementation(() => new Promise(() => {
        // Never resolves - intentionally empty
      }));
      service.startSync([mockPlaylist1], 'Music');

      const status = service.getSyncStatus();
      expect(status).toContain('Playlist 1');
      expect(status).toContain('%');
    });
  });

  describe('Computed Signals', () => {
    it('should compute percentage correctly', async () => {
      vi.clearAllMocks();
      vi.mocked(invoke).mockResolvedValue('Sync completed');

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');
      await vi.runAllTimersAsync();
      await syncPromise;

      expect(service.percentage()).toBe(100);
    });
  });
});

