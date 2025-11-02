export interface SyncProgress {
  isActive: boolean;
  currentPlaylist: string | null;
  completedPlaylists: number;
  totalPlaylists: number;
  percentage: number;
  status: string;
}

export interface OperationError {
  operation: string;
  error: string;
  category: string;
  isRetryable: boolean;
  attempts: number;
  filePath?: string;
  trackId?: string;
}

export interface SyncReport {
  success: boolean;
  totalOperations: number;
  successfulOperations: number;
  failedOperations: number;
  skippedOperations: number;
  errors: OperationError[];
  warnings: string[];
  durationMs: number;
  message: string;
}

export interface SyncResult {
  success: boolean;
  message: string;
  syncedPlaylists: string[];
  errors: string[];
  report?: SyncReport;
}
