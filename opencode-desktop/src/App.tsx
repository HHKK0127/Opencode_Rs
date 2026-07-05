import { useState } from 'react'
import { LoginPanel } from './components/LoginPanel'
import { FileUploadPanel } from './components/FileUploadPanel'
import { FileList } from './components/FileList'
import './styles/app.css'

const App = () => {
  const [isLoggedIn, setIsLoggedIn] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [fileRefresh, setFileRefresh] = useState(0)

  const handleLoginSuccess = () => {
    setIsLoggedIn(true)
    setError(null)
  }

  const handleUploadComplete = () => {
    // ファイル一覧を更新
    setFileRefresh(fileRefresh + 1)
  }

  if (!isLoggedIn) {
    return (
      <LoginPanel
        onLoginSuccess={handleLoginSuccess}
        onError={(err) => setError(err)}
      />
    )
  }

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>📁 OpenCode - ファイル管理</h1>
        <button
          onClick={() => {
            setIsLoggedIn(false)
            localStorage.removeItem('authToken')
          }}
          className="logout-button"
        >
          ログアウト
        </button>
      </header>

      <main className="app-main">
        {error && (
          <div className="error-toast">
            <span>{error}</span>
            <button onClick={() => setError(null)}>✕</button>
          </div>
        )}

        <div className="content-container">
          <section className="upload-section">
            <h2>📤 ファイルアップロード</h2>
            <FileUploadPanel
              onUploadComplete={handleUploadComplete}
              onError={(err) => setError(err)}
            />
          </section>

          <section className="list-section">
            <FileList
              refresh={fileRefresh}
              onError={(err) => setError(err)}
            />
          </section>
        </div>
      </main>
    </div>
  )
}

export default App

