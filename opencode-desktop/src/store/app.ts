import { create } from 'zustand'

export interface User {
  id: string
  username: string
}

export interface FileInfo {
  id: string
  filename: string
  size: number
  uploaded_at: string
  mime_type?: string
}

interface AppState {
  // Auth
  isAuthenticated: boolean
  user: User | null
  token: string | null

  // File management
  currentFile: string | null
  files: FileInfo[]
  selectedFile: FileInfo | null

  // UI state
  isDarkMode: boolean
  sidebarOpen: boolean
  isLoading: boolean
  error: string | null

  // Actions
  setAuthenticated(val: boolean): void
  setUser(user: User): void
  setToken(token: string): void
  setCurrentFile(path: string): void
  setFiles(files: FileInfo[]): void
  setSelectedFile(file: FileInfo | null): void
  toggleDarkMode(): void
  toggleSidebar(): void
  setLoading(val: boolean): void
  setError(error: string | null): void
  logout(): void
}

export const useAppStore = create<AppState>((set) => ({
  isAuthenticated: false,
  user: null,
  token: null,
  currentFile: null,
  files: [],
  selectedFile: null,
  isDarkMode: localStorage.getItem('darkMode') === 'true',
  sidebarOpen: true,
  isLoading: false,
  error: null,

  setAuthenticated: (val) => set({ isAuthenticated: val }),
  setUser: (user) => set({ user }),
  setToken: (token) => {
    localStorage.setItem('authToken', token)
    set({ token })
  },
  setCurrentFile: (path) => set({ currentFile: path }),
  setFiles: (files) => set({ files }),
  setSelectedFile: (file) => set({ selectedFile: file }),
  toggleDarkMode: () =>
    set((state) => {
      const newDarkMode = !state.isDarkMode
      localStorage.setItem('darkMode', String(newDarkMode))
      return { isDarkMode: newDarkMode }
    }),
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  setLoading: (val) => set({ isLoading: val }),
  setError: (error) => set({ error }),
  logout: () => {
    localStorage.removeItem('authToken')
    set({
      isAuthenticated: false,
      user: null,
      token: null,
      files: [],
      selectedFile: null,
      error: null,
    })
  },
}))
