import { Injectable, computed, inject } from '@angular/core';
import { MtpDeviceService } from './mtp-device.service';
import { ItunesLibraryService } from './itunes-library.service';
import { SyncService } from './sync.service';
import { DeviceInfo, DeviceConnectionState } from '../../shared/models/device.model';
import { LibraryState, Playlist } from '../../shared/models/library.model';
import { SyncProgress, SyncResult } from '../../shared/models/sync.model';

export interface AppState {
  device: DeviceConnectionState;
  library: LibraryState;
  sync: SyncProgress;
  canSync: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class AppStateService {
  // Inject services using inject() function for better zoneless compatibility
  private readonly mtpDeviceService = inject(MtpDeviceService);
  private readonly itunesLibraryService = inject(ItunesLibraryService);
  private readonly syncService = inject(SyncService);

  // Computed signals for derived state
  public readonly appState = computed<AppState>(() => {
    const deviceState = this.mtpDeviceService.connectionState();
    const libraryState = this.itunesLibraryService.libraryState();
    const syncProgress = this.syncService.syncProgress();

    const canSync = deviceState.isConnected &&
                   libraryState.isLoaded &&
                   !syncProgress.isActive &&
                   this.itunesLibraryService.getSelectedPlaylistCount() > 0;

    return {
      device: deviceState,
      library: libraryState,
      sync: syncProgress,
      canSync
    };
  });

  /**
   * Get the current app state
   */
  public getAppState(): AppState {
    return this.appState();
  }

  /**
   * Check if the app is ready to sync
   */
  public canSync(): boolean {
    return this.appState().canSync;
  }

  /**
   * Check if a device is connected
   */
  public isDeviceConnected(): boolean {
    return this.appState().device.isConnected;
  }

  /**
   * Check if a library is loaded
   */
  public isLibraryLoaded(): boolean {
    return this.appState().library.isLoaded;
  }

  /**
   * Check if sync is active
   */
  public isSyncActive(): boolean {
    return this.appState().sync.isActive;
  }

  /**
   * Get the connected device
   */
  public getConnectedDevice(): DeviceInfo | null {
    return this.appState().device.activeDevice;
  }

  /**
   * Get the loaded library
   */
  public getLoadedLibrary(): LibraryState {
    return this.appState().library;
  }

  /**
   * Get the current sync progress
   */
  public getSyncProgress(): SyncProgress {
    return this.appState().sync;
  }

  /**
   * Get any current errors
   */
  public getErrors(): { device: string | null; library: string | null } {
    const state = this.appState();
    return {
      device: state.device.error,
      library: state.library.error
    };
  }

  /**
   * Start syncing selected playlists
   */
  public async startSync(): Promise<SyncResult> {
    const selectedPlaylists = this.itunesLibraryService.selectedPlaylists();
    return await this.syncService.startSync(selectedPlaylists);
  }

  /**
   * Cancel current sync
   */
  public cancelSync(): void {
    this.syncService.cancelSync();
  }

  /**
   * Get sync status message
   */
  public getSyncStatusMessage(): string {
    return this.syncService.getSyncStatus();
  }

  /**
   * Get the number of selected playlists
   */
  public getSelectedPlaylistCount(): number {
    return this.itunesLibraryService.getSelectedPlaylistCount();
  }

  /**
   * Get all available playlists
   */
  public getPlaylists(): Playlist[] {
    return this.itunesLibraryService.getPlaylists();
  }

  /**
   * Toggle playlist selection
   */
  public togglePlaylistSelection(playlist: Playlist): void {
    this.itunesLibraryService.togglePlaylistSelection(playlist);
  }

  /**
   * Check if a playlist is selected
   */
  public isPlaylistSelected(playlist: Playlist): boolean {
    return this.itunesLibraryService.isPlaylistSelected(playlist);
  }

  /**
   * Select all playlists
   */
  public selectAllPlaylists(): void {
    this.itunesLibraryService.selectAllPlaylists();
  }

  /**
   * Deselect all playlists
   */
  public deselectAllPlaylists(): void {
    this.itunesLibraryService.deselectAllPlaylists();
  }
}
