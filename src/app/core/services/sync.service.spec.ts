import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi, afterEach, type Mock } from 'vitest';
import { installTauriInvokeMock } from '../tauri.testing';
import { SyncService } from './sync.service';
import { Playlist } from '../../shared/models/library.model';

describe('SyncService', () => {
  let service: SyncService;
  // Tauri IPC spy used by the service; (re)installed in beforeEach.
  let invoke: Mock;

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

  // `sync_playlist_to_device` returns a JSON-encoded SyncReport. Tests drive the
  // mock with this helper so the service's report parsing succeeds.
  const successReportJson = JSON.stringify({
    success: true,
    totalOperations: 1,
    successfulOperations: 1,
    failedOperations: 0,
    skippedOperations: 0,
    errors: [],
    warnings: [],
    durationMs: 1,
    message: 'Sync completed'
  });

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [SyncService]
    });

    // Install the Tauri IPC spy before the service is constructed.
    vi.clearAllMocks();
    invoke = installTauriInvokeMock();

    service = TestBed.inject(SyncService);
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
      invoke.mockResolvedValue(successReportJson);

      const syncPromise = service.startSync([mockPlaylist1], 'Music');
      await vi.runAllTimersAsync();
      const result = await syncPromise;

      expect(invoke).toHaveBeenCalledWith('sync_playlist_to_device', {
        playlistName: 'Playlist 1',
        deviceFolder: 'Music'
      });
      expect(result.success).toBe(true);
      expect(result.syncedPlaylists).toContain('Playlist 1');
      expect(result.errors.length).toBe(0);
    });

    it('should sync multiple playlists successfully', async () => {
      invoke.mockResolvedValue(successReportJson);

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');
      await vi.runAllTimersAsync();
      const result = await syncPromise;

      expect(invoke).toHaveBeenCalledTimes(2);
      expect(result.success).toBe(true);
      expect(result.syncedPlaylists.length).toBe(2);
    });

    it('should handle errors during sync', async () => {
      invoke.mockRejectedValue(new Error('Sync failed'));

      const syncPromise = service.startSync([mockPlaylist1], 'Music');
      await vi.runAllTimersAsync();
      const result = await syncPromise;

      expect(result.success).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.syncedPlaylists.length).toBe(0);
    });

    it('should handle partial failures', async () => {
      invoke
        .mockResolvedValueOnce(successReportJson) // First playlist succeeds
        .mockRejectedValueOnce(new Error('Sync failed')); // Second playlist fails

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');
      await vi.runAllTimersAsync();
      const result = await syncPromise;

      expect(result.success).toBe(false);
      expect(result.syncedPlaylists.length).toBe(1);
      expect(result.errors.length).toBe(1);
    });

    it('should update progress during sync', async () => {
      invoke.mockResolvedValue(successReportJson);

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');

      // Advance only the per-playlist delays (2 x 500ms) so the sync resolves
      // without firing the 3s progress-reset timer scheduled afterwards.
      await vi.advanceTimersByTimeAsync(2 * 500);
      await syncPromise;

      expect(service.syncProgress().completedPlaylists).toBe(2);
      expect(service.syncProgress().totalPlaylists).toBe(2);
      expect(service.syncProgress().percentage).toBe(100);
    });

    it('should use default folder when not specified', async () => {
      invoke.mockResolvedValue(successReportJson);

      const syncPromise = service.startSync([mockPlaylist1]);
      await vi.runAllTimersAsync();
      await syncPromise;

      expect(invoke).toHaveBeenCalledWith('sync_playlist_to_device', {
        playlistName: 'Playlist 1',
        deviceFolder: 'Music'
      });
    });
  });

  describe('cancelSync', () => {
    it('should cancel active sync', () => {
      // Start a sync (don't await to keep it active)
      invoke.mockImplementation(() => new Promise(() => {
        // Never resolves - intentionally empty
      }));
      service.startSync([mockPlaylist1], 'Music');

      service.cancelSync();

      expect(service.isActive()).toBe(false);
      expect(service.getSyncProgress().status).toContain('cancelled');
    });
  });

  describe('isSyncActive', () => {
    it('should return false when not active', () => {
      expect(service.isSyncActive()).toBe(false);
    });

    it('should return true when sync is active', async () => {
      invoke.mockImplementation(() => new Promise(() => {
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
      invoke.mockResolvedValue(successReportJson);
      const syncPromise = service.startSync([mockPlaylist1], 'Music');

      // Advance only the single per-playlist delay so the reset timer (3s) does
      // not fire and zero the progress before we assert.
      await vi.advanceTimersByTimeAsync(500);
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
      invoke.mockResolvedValue(successReportJson);
      const syncPromise = service.startSync([mockPlaylist1], 'Music');
      await vi.runAllTimersAsync();
      const result = await syncPromise;

      expect(service.getLastSyncResult()).toEqual(result);
    });
  });

  describe('clearSyncResult', () => {
    it('should clear sync result', async () => {
      invoke.mockResolvedValue(successReportJson);
      const syncPromise = service.startSync([mockPlaylist1], 'Music');
      await vi.runAllTimersAsync();
      await syncPromise;

      service.clearSyncResult();

      expect(service.getLastSyncResult()).toBeNull();
    });
  });

  describe('getSyncStatus', () => {
    it('should return ready message when not active', () => {
      expect(service.getSyncStatus()).toBe('Ready to sync');
    });

    it('should return status with playlist name during sync', async () => {
      invoke.mockImplementation(() => new Promise(() => {
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
      invoke.mockResolvedValue(successReportJson);

      const syncPromise = service.startSync([mockPlaylist1, mockPlaylist2], 'Music');
      // Advance only the per-playlist delays (2 x 500ms); leave the 3s reset.
      await vi.advanceTimersByTimeAsync(2 * 500);
      await syncPromise;

      expect(service.percentage()).toBe(100);
    });
  });
});

