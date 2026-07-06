import React, { useState } from 'react'
import { backendAPI } from '../services/backend-api'
import { useAppStore } from '../store/app'
import '../styles/LoginPanel.css'

interface LoginPanelProps {
  onLoginSuccess?: () => void
  onError?: (error: string) => void
}

export const LoginPanel: React.FC<LoginPanelProps> = ({ onLoginSuccess, onError }) => {
  const [username, setUsername] = useState('testuser')
  const [password, setPassword] = useState('testpassword')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { setAuthenticated, setUser, setToken } = useAppStore()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      setIsLoading(true)
      setError(null)

      const { token, user } = await backendAPI.login(username, password)

      setToken(token)
      setUser(user)
      setAuthenticated(true)
      onLoginSuccess?.()
    } catch (err) {
      const message = err instanceof Error ? err.message : 'ログイン失敗'
      setError(message)
      onError?.(message)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="login-panel">
      <div className="login-card">
        <h1 className="login-title">🔐 OpenCode</h1>
        <p className="login-subtitle">ログイン</p>

        <form onSubmit={handleSubmit} className="login-form">
          <div className="form-group">
            <label htmlFor="username">ユーザー名</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="ユーザー名を入力"
              disabled={isLoading}
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="password">パスワード</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="パスワードを入力"
              disabled={isLoading}
              required
            />
          </div>

          {error && <div className="form-error">❌ {error}</div>}

          <button
            type="submit"
            disabled={isLoading}
            className="login-button"
          >
            {isLoading ? 'ログイン中...' : 'ログイン'}
          </button>
        </form>

        <p className="demo-info">デモ: testuser / testpassword</p>
      </div>
    </div>
  )
}
