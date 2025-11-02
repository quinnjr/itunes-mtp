export interface SyncProgress {
  isActive: boolean;
  currentPlaylist: string | null;
  completedPlaylists: number;
  totalPlaylists: number;
  percentage: number;
  status: string;
}

export interface SyncResult {
  success: boolean;
  message: string;
  syncedPlaylists: string[];
  errors: string[];
}
