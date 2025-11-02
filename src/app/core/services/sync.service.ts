import { Injectable, signal, WritableSignal, computed } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { SyncProgress, SyncResult } from '../../shared/models/sync.model';
import { Playlist } from '../../shared/models/library.model';

@Injectable({
  providedIn: 'root'
})
export class SyncService {
  // Private writable signals
  private readonly _isActive = signal<boolean>(false);
  private readonly _currentPlaylist = signal<string | null>(null);
  private readonly _completedPlaylists = signal<number>(0);
  private readonly _totalPlaylists = signal<number>(0);
  private readonly _status = signal<string>('');
  private readonly _syncResult = signal<SyncResult | null>(null);

  // Public readonly signals
  public readonly syncResult = this._syncResult.asReadonly();

  // Computed signals for derived state
  public readonly syncProgress = computed<SyncProgress>(() => ({
    isActive: this._isActive(),
    currentPlaylist: this._currentPlaylist(),
    completedPlaylists: this._completedPlaylists(),
    totalPlaylists: this._totalPlaylists(),
    percentage: this._totalPlaylists() > 0
      ? Math.round((this._completedPlaylists() / this._totalPlaylists()) * 100)
      : 0,
    status: this._status()
  }));

  public readonly isActive = computed<boolean>(() => this._isActive());
  public readonly percentage = computed<number>(() =>
    this._totalPlaylists() > 0
      ? Math.round((this._completedPlaylists() / this._totalPlaylists()) * 100)
      : 0
  );

  constructor() {}

  /**
   * Start syncing selected playlists to the connected device
   */
  public async startSync(playlists: Playlist[], deviceFolder: string = 'Music'): Promise<SyncResult> {
    if (playlists.length === 0) {
      const result: SyncResult = {
        success: false,
        message: 'No playlists selected for sync',
        syncedPlaylists: [],
        errors: ['No playlists selected']
      };
      this._syncResult.set(result);
      return result;
    }

    this._startSyncProgress(playlists.length);
    const syncedPlaylists: string[] = [];
    const errors: string[] = [];

    try {
      for (let i = 0; i < playlists.length; i++) {
        const playlist = playlists[i];

        this._currentPlaylist.set(playlist.name);
        this._completedPlaylists.set(i);
        this._status.set(`Syncing ${playlist.name}...`);

        try {
          // Sync each playlist to the device
          const result = await invoke<string>('sync_playlist_to_device', {
            playlistName: playlist.name,
            deviceFolder
          });

          console.log(`Synced playlist "${playlist.name}":`, result);
          syncedPlaylists.push(playlist.name);

        } catch (error) {
          const errorMessage = `Failed to sync playlist "${playlist.name}": ${error}`;
          console.error(errorMessage);
          errors.push(errorMessage);
        }

        // Update progress
        this._completedPlaylists.set(i + 1);

        // Small delay to show progress
        await this._delay(500);
      }

      // Complete sync
      this._status.set('Sync completed');
      this._isActive.set(false);

      const result: SyncResult = {
        success: errors.length === 0,
        message: errors.length === 0
          ? `Successfully synced ${syncedPlaylists.length} playlist(s)`
          : `Synced ${syncedPlaylists.length} playlist(s) with ${errors.length} error(s)`,
        syncedPlaylists,
        errors
      };

      this._syncResult.set(result);

      // Reset progress after 3 seconds
      setTimeout(() => {
        this._resetSyncProgress();
      }, 3000);

      return result;

    } catch (error) {
      const errorMessage = `Sync failed: ${error}`;
      console.error(errorMessage);

      const result: SyncResult = {
        success: false,
        message: errorMessage,
        syncedPlaylists,
        errors: [...errors, errorMessage]
      };

      this._syncResult.set(result);
      this._isActive.set(false);

      return result;
    }
  }

  /**
   * Cancel the current sync operation
   */
  public cancelSync(): void {
    this._isActive.set(false);
    this._status.set('Sync cancelled');
  }

  /**
   * Check if sync is currently active
   */
  public isSyncActive(): boolean {
    return this.isActive();
  }

  /**
   * Get the current sync progress
   */
  public getSyncProgress(): SyncProgress {
    return this.syncProgress();
  }

  /**
   * Get the last sync result
   */
  public getLastSyncResult(): SyncResult | null {
    return this._syncResult();
  }

  /**
   * Clear the sync result
   */
  public clearSyncResult(): void {
    this._syncResult.set(null);
  }

  /**
   * Get sync status as a human-readable string
   */
  public getSyncStatus(): string {
    if (!this.isActive()) {
      return 'Ready to sync';
    }

    const currentPlaylist = this._currentPlaylist();
    const percentage = this.percentage();

    if (currentPlaylist) {
      return `Syncing ${currentPlaylist}... (${percentage}%)`;
    }

    return `Syncing... (${percentage}%)`;
  }

  // Private helper methods
  private _startSyncProgress(totalPlaylists: number): void {
    this._isActive.set(true);
    this._currentPlaylist.set(null);
    this._completedPlaylists.set(0);
    this._totalPlaylists.set(totalPlaylists);
    this._status.set('Starting sync...');
  }

  private _resetSyncProgress(): void {
    this._isActive.set(false);
    this._currentPlaylist.set(null);
    this._completedPlaylists.set(0);
    this._totalPlaylists.set(0);
    this._status.set('');
  }

  private _delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
