import { FileInfo, User } from '../store/app'

const API_BASE = 'http://localhost:8080/api/v1'

interface LoginResponse {
  status: string
  data: {
    token: string
    user: User
  }
}

interface FileResponse {
  status: string
  data: FileInfo
}

interface FilesListResponse {
  status: string
  data: {
    items: FileInfo[]
    total: number
    page: number
    per_page: number
  }
}

export class BackendAPI {
  private token: string | null = localStorage.getItem('authToken')

  setToken(token: string) {
    this.token = token
    localStorage.setItem('authToken', token)
  }

  getToken(): string | null {
    return this.token
  }

  private getHeaders(): HeadersInit {
    return {
      'Content-Type': 'application/json',
      ...(this.token && { Authorization: `Bearer ${this.token}` }),
    }
  }

  async login(username: string, password: string): Promise<{ token: string; user: User }> {
    const response = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ username, password }),
    })

    if (!response.ok) {
      throw new Error('Login failed')
    }

    const data: LoginResponse = await response.json()
    if (data.status === 'success') {
      this.setToken(data.data.token)
      return data.data
    }

    throw new Error('Invalid response format')
  }

  async register(username: string, password: string): Promise<{ token: string; user: User }> {
    const response = await fetch(`${API_BASE}/auth/register`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ username, password }),
    })

    if (!response.ok) {
      throw new Error('Registration failed')
    }

    const data: LoginResponse = await response.json()
    if (data.status === 'success') {
      this.setToken(data.data.token)
      return data.data
    }

    throw new Error('Invalid response format')
  }

  async refreshToken(): Promise<string> {
    const response = await fetch(`${API_BASE}/auth/refresh`, {
      method: 'POST',
      headers: this.getHeaders(),
    })

    if (!response.ok) {
      throw new Error('Token refresh failed')
    }

    const data: LoginResponse = await response.json()
    if (data.status === 'success') {
      this.setToken(data.data.token)
      return data.data.token
    }

    throw new Error('Invalid response format')
  }

  async uploadFile(file: File): Promise<FileInfo> {
    if (!this.token) {
      throw new Error('Not authenticated')
    }

    const formData = new FormData()
    formData.append('file', file)

    const response = await fetch(`${API_BASE}/files/upload`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${this.token}`,
      },
      body: formData,
    })

    if (!response.ok) {
      throw new Error('Upload failed')
    }

    const data: FileResponse = await response.json()
    if (data.status === 'success') {
      return data.data
    }

    throw new Error('Invalid response format')
  }

  async getFiles(page: number = 1, perPage: number = 50): Promise<{ items: FileInfo[]; total: number }> {
    const response = await fetch(`${API_BASE}/files?page=${page}&per_page=${perPage}`, {
      method: 'GET',
      headers: this.getHeaders(),
    })

    if (!response.ok) {
      throw new Error('Failed to fetch files')
    }

    const data: FilesListResponse = await response.json()
    if (data.status === 'success') {
      return {
        items: data.data.items,
        total: data.data.total,
      }
    }

    throw new Error('Invalid response format')
  }

  async deleteFile(fileId: string): Promise<void> {
    const response = await fetch(`${API_BASE}/files/${fileId}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    })

    if (!response.ok) {
      throw new Error('Delete failed')
    }
  }

  async downloadFile(fileId: string): Promise<Blob> {
    const response = await fetch(`${API_BASE}/files/${fileId}/download`, {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${this.token}`,
      },
    })

    if (!response.ok) {
      throw new Error('Download failed')
    }

    return await response.blob()
  }
}

export const backendAPI = new BackendAPI()
