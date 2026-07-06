import axios from 'axios'

const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'

// JWT トークンを保存
let authToken: string | null = null

// 認証情報取得
export const getAuthToken = () => authToken

// ログイン
export const login = async (username: string, password: string) => {
  const response = await axios.post(`${API_BASE}/api/v1/auth/login`, {
    username,
    password,
  }, {
    headers: { 'Content-Type': 'application/json' },
  })
  
  if (response.data?.data?.token) {
    authToken = response.data.data.token
  }
  return response.data
}

// API インスタンス (JWT 認証)
export const api = axios.create({
  baseURL: `${API_BASE}/api/v1`,
})

// リクエスト前に JWT token を追加
api.interceptors.request.use((config) => {
  if (authToken) {
    config.headers.Authorization = `Bearer ${authToken}`
  }
  return config
})

// V2 API (BasicAuth)
const AUTH = 'Basic ' + btoa('opencode:opencode')
export const apiV2 = axios.create({
  baseURL: `${API_BASE}/v2`,
  headers: {
    'Authorization': AUTH,
    'Content-Type': 'application/json',
  },
})

export const createSession = async () => {
  const res = await apiV2.post('/session')
  return res.data
}

export const sendPrompt = async (sessionId: string, prompt: string) => {
  const res = await apiV2.post(`/session/${sessionId}/prompt`, { prompt })
  return res.data
}

export const fetchMessages = async (sessionId: string) => {
  const res = await apiV2.get(`/session/${sessionId}/messages`)
  return res.data
}

// ファイル API (V1)
export interface UploadInitRequest {
  file_name: string
  file_size: number
  mime_type?: string
  chunk_size?: number
}

export interface UploadInitResponse {
  session_id: string
  chunk_size: number
  expected_chunks: number
}

export interface FileListItem {
  id: string
  filename: string
  size: number
  mime_type: string
  uploaded_at: string
}

// ファイルアップロード セッション初期化
export const initializeFileUpload = async (request: UploadInitRequest): Promise<UploadInitResponse> => {
  const res = await api.post('/files/upload/init', request)
  return res.data?.data || res.data
}

// チャンクアップロード
export const uploadFileChunk = async (
  sessionId: string,
  chunkIndex: number,
  chunkData: Blob
): Promise<{ progress_percent: number }> => {
  const formData = new FormData()
  formData.append('session_id', sessionId)
  formData.append('chunk_index', chunkIndex.toString())
  formData.append('chunk', chunkData)

  const res = await api.post('/files/upload/chunk', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  })
  return res.data?.data || res.data
}

// ファイルアップロード完了
export const completeFileUpload = async (
  sessionId: string,
  checksum?: string
): Promise<{ file_id: string; filename: string; size: number }> => {
  const res = await api.post(`/files/upload/complete/${sessionId}`, {
    checksum: checksum || 'auto',
  })
  return res.data?.data || res.data
}

// アップロード進捗確認
export const getUploadProgress = async (
  sessionId: string
): Promise<{ progress_percent: number }> => {
  const res = await api.get(`/files/upload/progress/${sessionId}`)
  return res.data?.data || res.data
}

// ファイル一覧取得
export const listFiles = async (page = 1, perPage = 20): Promise<FileListItem[]> => {
  const res = await api.get('/files', {
    params: { page, per_page: perPage },
  })
  return res.data?.data || []
}

// ファイルメタデータ取得
export const getFileMetadata = async (fileId: string) => {
  const res = await api.get(`/files/${fileId}`)
  return res.data?.data || res.data
}

// ファイルダウンロード
export const downloadFile = (fileId: string) => {
  return `${API_BASE}/api/v1/files/${fileId}/download`
}
