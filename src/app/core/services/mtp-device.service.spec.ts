import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { MtpDeviceService } from './mtp-device.service';
import { AsyncHandlerService } from './async-handler.service';
import { DeviceInfo, FileInfo } from '../../shared/models/device.model';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('MtpDeviceService', () => {
  let service: MtpDeviceService;

  const mockDevice: DeviceInfo = {
    device_id: 'test-device-123',
    friendly_name: 'Test Device',
    manufacturer: 'Test Manufacturer'
  };

  const mockFile: FileInfo = {
    object_id: 'file-123',
    name: 'test.mp3',
    size: 1024,
    is_folder: false
  };

  const mockFolder: FileInfo = {
    object_id: 'folder-123',
    name: 'Music',
    size: 0,
    is_folder: true
  };

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [MtpDeviceService, AsyncHandlerService]
    });

    service = TestBed.inject(MtpDeviceService);

    // Reset mocks
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue([]);
  });

  describe('Service Initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });

    it('should initialize with empty device list', () => {
      expect(service.devices().length).toBe(0);
    });

    it('should initialize with no active device', () => {
      expect(service.isConnected()).toBe(false);
      expect(service.activeDevice()).toBeNull();
    });

    it('should initialize with no error', () => {
      expect(service.error()).toBeNull();
    });

    it('should initialize with not loading', () => {
      expect(service.isLoading()).toBe(false);
    });
  });

  describe('refreshDevices', () => {
    it('should load devices successfully', async () => {
      const mockDevices: DeviceInfo[] = [mockDevice];
      vi.mocked(invoke).mockResolvedValue(mockDevices);

      await service.refreshDevices();

      expect(invoke).toHaveBeenCalledWith('get_devices');
      expect(service.devices().length).toBe(1);
      expect(service.devices()[0]).toEqual(mockDevice);
    });

    it('should handle empty device list', async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      await service.refreshDevices();

      expect(service.devices().length).toBe(0);
    });

    it('should handle errors when loading devices', async () => {
      const errorMessage = 'Failed to get devices';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      await service.refreshDevices();

      expect(service.error()).toBe(errorMessage);
      expect(service.devices().length).toBe(0);
    });
  });

  describe('connectToDevice', () => {
    beforeEach(() => {
      vi.mocked(invoke).mockResolvedValue(undefined);
    });

    it('should connect to device successfully', async () => {
      await service.connectToDevice(mockDevice);

      expect(invoke).toHaveBeenCalledWith('connect_device', {
        deviceId: mockDevice.device_id
      });
      expect(service.activeDevice()).toEqual(mockDevice);
      expect(service.isConnected()).toBe(true);
    });

    it('should load device files after connection', async () => {
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');
      vi.mocked(invoke).mockResolvedValueOnce(undefined); // connect_device
      vi.mocked(invoke).mockResolvedValueOnce([]); // list_device_files

      await service.connectToDevice(mockDevice);

      expect(loadFilesSpy).toHaveBeenCalled();
    });

    it('should handle connection errors', async () => {
      const errorMessage = 'Connection failed';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      await service.connectToDevice(mockDevice);

      expect(service.error()).toBe(errorMessage);
      expect(service.activeDevice()).toBeNull();
    });
  });

  describe('disconnectDevice', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should disconnect device successfully', async () => {
      await service.disconnectDevice();

      expect(invoke).toHaveBeenCalledWith('disconnect_device');
      expect(service.activeDevice()).toBeNull();
      expect(service.deviceFiles().length).toBe(0);
      expect(service.currentFolder()).toBeNull();
    });

    it('should handle disconnect errors gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Disconnect failed'));

      await service.disconnectDevice();

      // Should still clear local state even on error
      expect(service.activeDevice()).toBeNull();
    });
  });

  describe('loadDeviceFiles', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should load files successfully', async () => {
      const mockFiles: FileInfo[] = [mockFile, mockFolder];
      vi.mocked(invoke).mockResolvedValue(mockFiles);

      await service.loadDeviceFiles();

      expect(invoke).toHaveBeenCalledWith('list_device_files', { folderId: undefined });
      expect(service.deviceFiles().length).toBe(2);
    });

    it('should load files from specific folder', async () => {
      const folderId = 'folder-123';
      const mockFiles: FileInfo[] = [mockFile];
      vi.mocked(invoke).mockResolvedValue(mockFiles);

      await service.loadDeviceFiles(folderId);

      expect(invoke).toHaveBeenCalledWith('list_device_files', { folderId });
      expect(service.currentFolder()).toBe(folderId);
    });

    it('should not load files if no device connected', async () => {
      await service.disconnectDevice();

      await service.loadDeviceFiles();

      expect(invoke).not.toHaveBeenCalledWith('list_device_files', expect.any(Object));
    });

    it('should handle errors when loading files', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to load files'));

      await service.loadDeviceFiles();

      expect(service.deviceFiles().length).toBe(0);
    });
  });

  describe('browseFolder', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should browse into folder', async () => {
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');
      vi.mocked(invoke).mockResolvedValue([]);

      await service.browseFolder(mockFolder);

      expect(loadFilesSpy).toHaveBeenCalledWith(mockFolder.object_id);
    });

    it('should not browse if file is not a folder', async () => {
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');

      await service.browseFolder(mockFile);

      expect(loadFilesSpy).not.toHaveBeenCalled();
    });
  });

  describe('goUpFolder', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should go up one folder level', async () => {
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');
      vi.mocked(invoke).mockResolvedValue([]);

      await service.goUpFolder();

      expect(loadFilesSpy).toHaveBeenCalledWith(undefined);
    });
  });

  describe('getFileInfo', () => {
    it('should get file info successfully', async () => {
      vi.mocked(invoke).mockResolvedValue(mockFile);

      const result = await service.getFileInfo('file-123');

      expect(invoke).toHaveBeenCalledWith('get_file_info', { objectId: 'file-123' });
      expect(result).toEqual(mockFile);
    });

    it('should return null on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('File not found'));

      const result = await service.getFileInfo('invalid-id');

      expect(result).toBeNull();
    });
  });

  describe('transferFile', () => {
    it('should transfer file successfully', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const result = await service.transferFile('file-123', 'C:\\dest\\file.mp3');

      expect(invoke).toHaveBeenCalledWith('transfer_file', {
        objectId: 'file-123',
        destPath: 'C:\\dest\\file.mp3'
      });
      expect(result).toBe(true);
    });

    it('should return false on error', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Transfer failed'));

      const result = await service.transferFile('file-123', 'C:\\dest\\file.mp3');

      expect(result).toBe(false);
    });
  });

  describe('createFolder', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should create folder successfully', async () => {
      const folderId = 'new-folder-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(folderId) // create_folder
        .mockResolvedValueOnce([]); // loadDeviceFiles

      const result = await service.createFolder('parent-123', 'NewFolder');

      expect(invoke).toHaveBeenCalledWith('create_folder', {
        parentId: 'parent-123',
        folderName: 'NewFolder'
      });
      expect(result).toBe(folderId);
    });

    it('should refresh device files after folder creation', async () => {
      const folderId = 'new-folder-123';
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');
      vi.mocked(invoke)
        .mockResolvedValueOnce(folderId)
        .mockResolvedValueOnce([]);

      await service.createFolder('parent-123', 'NewFolder');

      expect(loadFilesSpy).toHaveBeenCalled();
    });

    it('should throw error if no device connected', async () => {
      await service.disconnectDevice();

      await expect(
        service.createFolder('parent-123', 'NewFolder')
      ).rejects.toThrow('No device connected');
    });

    it('should handle errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to create folder'));

      await expect(
        service.createFolder('parent-123', 'NewFolder')
      ).rejects.toThrow('Failed to create folder');
    });

    it('should handle empty parent ID', async () => {
      const folderId = 'new-folder-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(folderId)
        .mockResolvedValueOnce([]);

      await service.createFolder('', 'NewFolder');

      expect(invoke).toHaveBeenCalledWith('create_folder', {
        parentId: '',
        folderName: 'NewFolder'
      });
    });

    it('should handle special characters in folder name', async () => {
      const folderId = 'special-folder-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(folderId)
        .mockResolvedValueOnce([]);

      await service.createFolder('parent-123', 'Folder & Name (2024)');

      expect(invoke).toHaveBeenCalledWith('create_folder', {
        parentId: 'parent-123',
        folderName: 'Folder & Name (2024)'
      });
    });
  });

  describe('ensureFolderPath', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should ensure folder path successfully', async () => {
      const folderId = 'final-folder-123';
      vi.mocked(invoke).mockResolvedValue(folderId);

      const result = await service.ensureFolderPath('base-123', 'Music/Artist/Album');

      expect(invoke).toHaveBeenCalledWith('ensure_folder_path', {
        baseFolderId: 'base-123',
        path: 'Music/Artist/Album'
      });
      expect(result).toBe(folderId);
    });

    it('should handle single-level path', async () => {
      const folderId = 'music-folder-123';
      vi.mocked(invoke).mockResolvedValue(folderId);

      const result = await service.ensureFolderPath('base-123', 'Music');

      expect(invoke).toHaveBeenCalledWith('ensure_folder_path', {
        baseFolderId: 'base-123',
        path: 'Music'
      });
      expect(result).toBe(folderId);
    });

    it('should handle nested paths', async () => {
      const folderId = 'deep-folder-123';
      vi.mocked(invoke).mockResolvedValue(folderId);

      const result = await service.ensureFolderPath('base-123', 'Music/Artist/Album/Year');

      expect(invoke).toHaveBeenCalledWith('ensure_folder_path', {
        baseFolderId: 'base-123',
        path: 'Music/Artist/Album/Year'
      });
      expect(result).toBe(folderId);
    });

    it('should throw error if no device connected', async () => {
      await service.disconnectDevice();

      await expect(
        service.ensureFolderPath('base-123', 'Music/Artist')
      ).rejects.toThrow('No device connected');
    });

    it('should handle errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Path creation failed'));

      await expect(
        service.ensureFolderPath('base-123', 'Music/Artist')
      ).rejects.toThrow('Failed to ensure folder path');
    });

    it('should handle empty path', async () => {
      const baseId = 'base-123';
      vi.mocked(invoke).mockResolvedValue(baseId);

      const result = await service.ensureFolderPath(baseId, '');

      expect(invoke).toHaveBeenCalledWith('ensure_folder_path', {
        baseFolderId: baseId,
        path: ''
      });
      expect(result).toBe(baseId);
    });
  });

  describe('getOrCreateMusicFolder', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should get or create Music folder successfully', async () => {
      const musicFolderId = 'music-folder-123';
      vi.mocked(invoke).mockResolvedValue(musicFolderId);

      const result = await service.getOrCreateMusicFolder();

      expect(invoke).toHaveBeenCalledWith('get_or_create_music_folder');
      expect(result).toBe(musicFolderId);
    });

    it('should return existing Music folder if already exists', async () => {
      const existingFolderId = 'existing-music-123';
      vi.mocked(invoke).mockResolvedValue(existingFolderId);

      const result1 = await service.getOrCreateMusicFolder();
      const result2 = await service.getOrCreateMusicFolder();

      expect(result1).toBe(existingFolderId);
      expect(result2).toBe(existingFolderId);
      expect(invoke).toHaveBeenCalledTimes(2);
    });

    it('should throw error if no device connected', async () => {
      await service.disconnectDevice();

      await expect(
        service.getOrCreateMusicFolder()
      ).rejects.toThrow('No device connected');
    });

    it('should handle errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Music folder creation failed'));

      await expect(
        service.getOrCreateMusicFolder()
      ).rejects.toThrow('Failed to get or create Music folder');
    });
  });

  describe('uploadFile', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should upload file successfully', async () => {
      const objectId = 'uploaded-file-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId) // upload_file
        .mockResolvedValueOnce([]); // loadDeviceFiles

      const result = await service.uploadFile(
        'C:\\Music\\song.mp3',
        'parent-123',
        'song.mp3'
      );

      expect(invoke).toHaveBeenCalledWith('upload_file', {
        localPath: 'C:\\Music\\song.mp3',
        parentFolderId: 'parent-123',
        fileName: 'song.mp3'
      });
      expect(result).toBe(objectId);
    });

    it('should refresh device files after upload', async () => {
      const objectId = 'uploaded-file-123';
      const loadFilesSpy = vi.spyOn(service, 'loadDeviceFiles');
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId)
        .mockResolvedValueOnce([]);

      await service.uploadFile('C:\\song.mp3', 'parent-123', 'song.mp3');

      expect(loadFilesSpy).toHaveBeenCalledWith('parent-123');
    });

    it('should throw error if no device connected', async () => {
      await service.disconnectDevice();

      await expect(
        service.uploadFile('C:\\song.mp3', 'parent-123', 'song.mp3')
      ).rejects.toThrow('No device connected');
    });

    it('should handle upload errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Upload failed'));

      await expect(
        service.uploadFile('C:\\song.mp3', 'parent-123', 'song.mp3')
      ).rejects.toThrow('Failed to upload file');
    });

    it('should handle different file paths', async () => {
      const objectId = 'uploaded-file-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId)
        .mockResolvedValueOnce([]);

      // Network path
      await service.uploadFile('\\\\server\\share\\song.mp3', 'parent-123', 'song.mp3');
      expect(invoke).toHaveBeenCalledWith('upload_file', {
        localPath: '\\\\server\\share\\song.mp3',
        parentFolderId: 'parent-123',
        fileName: 'song.mp3'
      });

      // Different drive
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId)
        .mockResolvedValueOnce([]);
      await service.uploadFile('D:\\Music\\song.mp3', 'parent-123', 'song.mp3');
      expect(invoke).toHaveBeenCalledWith('upload_file', {
        localPath: 'D:\\Music\\song.mp3',
        parentFolderId: 'parent-123',
        fileName: 'song.mp3'
      });
    });

    it('should handle different file types', async () => {
      const objectId = 'uploaded-file-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId)
        .mockResolvedValueOnce([]);

      // Audio file
      await service.uploadFile('C:\\song.mp3', 'parent-123', 'song.mp3');

      // Image file
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId)
        .mockResolvedValueOnce([]);
      await service.uploadFile('C:\\cover.jpg', 'parent-123', 'cover.jpg');

      expect(invoke).toHaveBeenCalledTimes(4); // 2 uploads + 2 loadDeviceFiles
    });

    it('should handle empty file name', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Invalid file name'));

      await expect(
        service.uploadFile('C:\\song.mp3', 'parent-123', '')
      ).rejects.toThrow();
    });

    it('should handle loadDeviceFiles error gracefully', async () => {
      const objectId = 'uploaded-file-123';
      vi.mocked(invoke)
        .mockResolvedValueOnce(objectId) // upload succeeds
        .mockRejectedValueOnce(new Error('Failed to load files')); // loadDeviceFiles fails

      // Should still return the object ID even if refresh fails
      const result = await service.uploadFile('C:\\song.mp3', 'parent-123', 'song.mp3');
      expect(result).toBe(objectId);
    });
  });

  describe('getActiveDevice', () => {
    it('should return null when no device connected', () => {
      expect(service.getActiveDevice()).toBeNull();
    });

    it('should return active device when connected', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);

      expect(service.getActiveDevice()).toEqual(mockDevice);
    });
  });

  describe('getError', () => {
    it('should return null when no error', () => {
      expect(service.getError()).toBeNull();
    });

    it('should return error message when error exists', async () => {
      const errorMessage = 'Test error';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      await service.refreshDevices();

      expect(service.getError()).toBe(errorMessage);
    });
  });

  describe('Computed Signals', () => {
    it('should compute connectionState correctly', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      expect(service.connectionState().isConnected).toBe(false);

      await service.connectToDevice(mockDevice);

      expect(service.connectionState().isConnected).toBe(true);
      expect(service.connectionState().activeDevice).toEqual(mockDevice);
    });

    it('should update connectionState on error', async () => {
      const errorMessage = 'Connection error';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      await service.connectToDevice(mockDevice);

      expect(service.connectionState().error).toBe(errorMessage);
    });
  });

  describe('Edge Cases and Integration', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      await service.connectToDevice(mockDevice);
    });

    it('should handle multiple folder creations in sequence', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('folder-1')
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce('folder-2')
        .mockResolvedValueOnce([]);

      const folder1 = await service.createFolder('parent', 'Folder1');
      const folder2 = await service.createFolder('parent', 'Folder2');

      expect(folder1).toBe('folder-1');
      expect(folder2).toBe('folder-2');
      expect(invoke).toHaveBeenCalledTimes(4);
    });

    it('should handle folder creation and file upload workflow', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('music-folder-123')
        .mockResolvedValueOnce('artist-folder-123')
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce('uploaded-file-123')
        .mockResolvedValueOnce([]);

      const musicFolder = await service.getOrCreateMusicFolder();
      const artistFolder = await service.ensureFolderPath(musicFolder, 'Artist Name');
      const fileId = await service.uploadFile('C:\\song.mp3', artistFolder, 'song.mp3');

      expect(musicFolder).toBe('music-folder-123');
      expect(artistFolder).toBe('artist-folder-123');
      expect(fileId).toBe('uploaded-file-123');
    });

    it('should handle connection loss during operation', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined); // connect

      // Simulate connection loss
      vi.spyOn(service, 'isConnected').mockReturnValue(false);

      await expect(
        service.createFolder('parent', 'Folder')
      ).rejects.toThrow('No device connected');
    });

    it('should maintain state consistency after errors', async () => {
      const initialDeviceCount = service.devices().length;
      vi.mocked(invoke).mockRejectedValue(new Error('Operation failed'));

      try {
        await service.refreshDevices();
      } catch {
        // Expected error - operation fails
      }

      // State should be reset on error
      expect(service.devices().length).toBe(initialDeviceCount);
      expect(service.error()).toBeTruthy();
    });
  });
});

