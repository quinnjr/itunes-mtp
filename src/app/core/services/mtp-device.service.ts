import { Injectable, signal, computed, inject } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { DeviceInfo, FileInfo, DeviceConnectionState } from '../../shared/models/device.model';
import { AsyncHandlerService } from './async-handler.service';

@Injectable({
  providedIn: 'root'
})
export class MtpDeviceService {
  // Inject async handler service
  private readonly asyncHandler = inject(AsyncHandlerService);

  // Private writable signals
  private readonly _devices = signal<DeviceInfo[]>([]);
  private readonly _isLoading = signal<boolean>(false);
  private readonly _error = signal<string | null>(null);
  private readonly _activeDevice = signal<DeviceInfo | null>(null);
  private readonly _deviceFiles = signal<FileInfo[]>([]);
  private readonly _currentFolder = signal<string | null>(null);

  // Public readonly signals
  public readonly devices = this._devices.asReadonly();
  public readonly deviceFiles = this._deviceFiles.asReadonly();
  public readonly currentFolder = this._currentFolder.asReadonly();

  // Computed signals for derived state
  public readonly connectionState = computed<DeviceConnectionState>(() => ({
    isConnected: this._activeDevice() !== null,
    activeDevice: this._activeDevice(),
    isLoading: this._isLoading(),
    error: this._error()
  }));

  public readonly isConnected = computed<boolean>(() => this._activeDevice() !== null);
  public readonly isLoading = computed<boolean>(() => this._isLoading());
  public readonly error = computed<string | null>(() => this._error());
  public readonly activeDevice = computed<DeviceInfo | null>(() => this._activeDevice());

  constructor() {
    // Initialize by loading devices
    this.refreshDevices();
  }

  /**
   * Refresh the list of available MTP devices
   */
  public async refreshDevices(): Promise<void> {
    await this.asyncHandler.executeAsync(
      async () => {
        const deviceList = await invoke<DeviceInfo[]>('get_devices');
        console.log('Devices loaded:', deviceList.length);
        return deviceList;
      },
      {
        setLoading: (loading) => this._isLoading.set(loading),
        setData: (devices) => this._devices.set(devices),
        setError: (error) => {
          this._error.set(error);
          if (error) this._devices.set([]);
        }
      }
    );
  }

  /**
   * Connect to a specific MTP device
   */
  public async connectToDevice(device: DeviceInfo): Promise<void> {
    await this.asyncHandler.executeAsync(
      async () => {
        await invoke('connect_device', { deviceId: device.device_id });
        console.log('Connected to device:', device.friendly_name);
        return device;
      },
      {
        setLoading: (loading) => this._isLoading.set(loading),
        setData: (connectedDevice) => {
          this._activeDevice.set(connectedDevice);
          // Load device files after successful connection
          this.loadDeviceFiles();
        },
        setError: (error) => this._error.set(error)
      }
    );
  }

  /**
   * Disconnect from the current device
   */
  public async disconnectDevice(): Promise<void> {
    try {
      await invoke('disconnect_device');

      this._activeDevice.set(null);
      this._deviceFiles.set([]);
      this._currentFolder.set(null);

      console.log('Device disconnected');
    } catch (error) {
      console.error('Failed to disconnect device:', error);
    }
  }

  /**
   * Load files from the connected device
   */
  public async loadDeviceFiles(folderId?: string): Promise<void> {
    if (!this.isConnected()) {
      console.warn('No device connected');
      return;
    }

    await this.asyncHandler.executeAsync(
      async () => {
        const files = await invoke<FileInfo[]>('list_device_files', { folderId });
        console.log('Device files loaded:', files.length);
        return files;
      },
      {
        setLoading: () => {
          // No loading state for file loading - intentionally empty
        },
        setData: (files) => {
          this._deviceFiles.set(files);
          this._currentFolder.set(folderId || null);
        },
        setError: (error) => {
          console.error('Failed to load device files:', error);
          this._deviceFiles.set([]);
        }
      }
    );
  }

  /**
   * Browse into a folder
   */
  public async browseFolder(file: FileInfo): Promise<void> {
    if (file.is_folder) {
      await this.loadDeviceFiles(file.object_id);
    }
  }

  /**
   * Go up one folder level
   */
  public async goUpFolder(): Promise<void> {
    await this.loadDeviceFiles();
  }

  /**
   * Get detailed information about a specific file
   */
  public async getFileInfo(objectId: string): Promise<FileInfo | null> {
    try {
      const fileInfo = await invoke<FileInfo>('get_file_info', { objectId });
      return fileInfo;
    } catch (error) {
      console.error('Failed to get file info:', error);
      return null;
    }
  }

  /**
   * Transfer a file from device to local storage
   */
  public async transferFile(objectId: string, destPath: string): Promise<boolean> {
    try {
      await invoke('transfer_file', { objectId, destPath });
      console.log('File transferred successfully:', destPath);
      return true;
    } catch (error) {
      console.error('Failed to transfer file:', error);
      return false;
    }
  }

  /**
   * Get the currently connected device
   */
  public getActiveDevice(): DeviceInfo | null {
    return this.activeDevice();
  }

  /**
   * Get the current error state
   */
  public getError(): string | null {
    return this.error();
  }

  /**
   * Create a folder on the connected device
   * @param parentId Object ID of the parent folder
   * @param folderName Name of the folder to create
   * @returns Object ID of the created folder
   */
  public async createFolder(parentId: string, folderName: string): Promise<string> {
    if (!this.isConnected()) {
      throw new Error('No device connected');
    }

    try {
      const folderId = await invoke<string>('create_folder', {
        parentId,
        folderName
      });
      console.log('Folder created:', folderName, 'with ID:', folderId);

      // Refresh device files to show the new folder
      await this.loadDeviceFiles(this._currentFolder() || undefined);

      return folderId;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to create folder:', errorMessage);
      throw new Error(`Failed to create folder: ${errorMessage}`);
    }
  }

  /**
   * Ensure a folder path exists on the device, creating all necessary parent folders
   * @param baseFolderId Object ID of the base folder to start from
   * @param path Folder path (e.g., "Music/Artist Name/Album Name")
   * @returns Object ID of the final folder
   */
  public async ensureFolderPath(baseFolderId: string, path: string): Promise<string> {
    if (!this.isConnected()) {
      throw new Error('No device connected');
    }

    try {
      const folderId = await invoke<string>('ensure_folder_path', {
        baseFolderId,
        path
      });
      console.log('Folder path ensured:', path, 'with final folder ID:', folderId);
      return folderId;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to ensure folder path:', errorMessage);
      throw new Error(`Failed to ensure folder path: ${errorMessage}`);
    }
  }

  /**
   * Get or create the base Music folder on the device
   * @returns Object ID of the Music folder
   */
  public async getOrCreateMusicFolder(): Promise<string> {
    if (!this.isConnected()) {
      throw new Error('No device connected');
    }

    try {
      const musicFolderId = await invoke<string>('get_or_create_music_folder');
      console.log('Music folder ID:', musicFolderId);
      return musicFolderId;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to get or create Music folder:', errorMessage);
      throw new Error(`Failed to get or create Music folder: ${errorMessage}`);
    }
  }

  /**
   * Upload a file from local filesystem to the connected device
   * @param localPath Path to the local file to upload
   * @param parentFolderId Object ID of the parent folder on the device
   * @param fileName Name to use for the file on the device
   * @returns Object ID of the uploaded file
   */
  public async uploadFile(localPath: string, parentFolderId: string, fileName: string): Promise<string> {
    if (!this.isConnected()) {
      throw new Error('No device connected');
    }

    try {
      const objectId = await invoke<string>('upload_file', {
        localPath,
        parentFolderId,
        fileName
      });
      console.log('File uploaded successfully:', fileName, 'with object ID:', objectId);

      // Refresh device files to show the new file
      await this.loadDeviceFiles(parentFolderId);

      return objectId;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to upload file:', errorMessage);
      throw new Error(`Failed to upload file: ${errorMessage}`);
    }
  }

  /**
   * Get storage capacity information for the connected device
   * @returns StorageInfo with total, free, and used space in bytes
   */
  public async getStorageInfo(): Promise<{ totalSpace: number; freeSpace: number; usedSpace: number }> {
    if (!this.isConnected()) {
      throw new Error('No device connected');
    }

    try {
      const storageInfo = await invoke<{ total_space: number; free_space: number; used_space: number }>('get_device_storage_info');
      console.log('Storage info retrieved:', storageInfo);
      return {
        totalSpace: storageInfo.total_space,
        freeSpace: storageInfo.free_space,
        usedSpace: storageInfo.used_space
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to get storage info:', errorMessage);
      throw new Error(`Failed to get storage info: ${errorMessage}`);
    }
  }
}
