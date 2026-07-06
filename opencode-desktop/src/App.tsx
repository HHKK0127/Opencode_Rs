import React, { useEffect } from 'react'
import { LoginPanel } from './components/LoginPanel'
import { FileManager } from './components/FileManager'
import { StatusBar } from './components/StatusBar'
import { useAppStore } from './store/app'
import { backendAPI } from './services/backend-api'
import './styles/app.css'

const App: React.FC = () => {
  const { isAuthenticated, isDarkMode, toggleDarkMode, logout, setError, isLoading, setLoading } = useAppStore()

  useEffect(() => {
    // ダークモード適用
    if (isDarkMode) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDarkMode])

  useEffect(() => {
    // 起動時にトークン確認
    const initApp = async () => {
      try {
        setLoading(true)
        const token = backendAPI.getToken()
        if (!token) {
          setLoading(false)
          return
        }
        
        // トークン有効性確認
        await backendAPI.refreshToken()
      } catch (err) {
        logout()
      } finally {
        setLoading(false)
      }
    }

    initApp()
  }, [logout, setLoading])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-white dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">ロード中...</p>
        </div>
      </div>
    )
  }

  if (!isAuthenticated) {
    return <LoginPanel />
  }

  return (
    <div className="flex flex-col h-screen bg-white dark:bg-gray-900 transition-colors duration-200">
      {/* ヘッダー */}
      <header className="flex items-center justify-between px-4 py-2 
                         bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 rounded-t-lg">
        <div className="flex items-center space-x-3">
          <h1 className="text-lg font-bold text-gray-900 dark:text-white">OpenCode</h1>
          <span className="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded">
            POC
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={toggleDarkMode}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            title="ダークモード切り替え"
          >
            {isDarkMode ? (
              <svg className="w-5 h-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"
                  clipRule="evenodd"
                />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
              </svg>
            )}
          </button>

          <button
            onClick={logout}
            className="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
          >
            ログアウト
          </button>
        </div>
      </header>

      {/* メインコンテンツ */}
      <div className="flex flex-1 overflow-hidden">
        <aside className="w-64 flex-shrink-0">
          <FileManager />
        </aside>

        <main className="flex-1 flex flex-col min-w-0">
          {/* エディタエリア（プレースホルダー） */}
          <div className="flex-1 flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <div className="text-center text-gray-400 dark:text-gray-600">
              <svg
                className="w-16 h-16 mx-auto mb-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1}
                  d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
                />
              </svg>
              <p className="text-lg font-medium">ファイルを選択してください</p>
              <p className="text-sm mt-1">ドラッグ&ドロップでアップロード</p>
            </div>
          </div>
        </main>
      </div>

      {/* ステータスバー */}
      <StatusBar />
    </div>
  )
}

export default App
