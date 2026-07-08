// ============================================================
// Response Envelope pattern support
// OpenCode_Rs backend uses the following format for all APIs:
// { status: "success" | "error", data: T | null, error: string | null, timestamp: string }
// However, error responses may use { code: number, error: string } format, so both are supported
// ============================================================

export interface ApiResponse<T> {
  status: 'success' | 'error';
  data: T | null;
  error: string | null;
  timestamp?: string;
}

export interface ApiError {
  code?: number;
  error: string;
}

export interface LoginData {
  token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: string;
    username: string;
  };
}

export interface RefreshData {
  token: string;
  expires_in: number;
}

export interface FileItem {
  id: string;
  filename: string;
  size: number;
  mime_type: string;
  created_at: string;
  updated_at: string;
  user_id: string;
}

export interface FilesData {
  files: FileItem[];
  total: number;
  page: number;
  per_page: number;
}

/** @deprecated Use ApiResponse<LoginData> instead */
export type LoginResponse = ApiResponse<LoginData>;

/** @deprecated Use ApiResponse<FilesData> instead */
export type FilesResponse = ApiResponse<FilesData>;

/** @deprecated Use ApiResponse<RefreshData> instead */
export type RefreshResponse = ApiResponse<RefreshData>;