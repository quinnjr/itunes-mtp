export interface DeviceInfo {
  device_id: string;
  friendly_name: string;
  manufacturer: string;
}

export interface FileInfo {
  object_id: string;
  name: string;
  size: number;
  is_folder: boolean;
}

export interface DeviceConnectionState {
  isConnected: boolean;
  activeDevice: DeviceInfo | null;
  isLoading: boolean;
  error: string | null;
}
