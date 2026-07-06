import React, { useState, useCallback, useEffect } from 'react'
import { FileInfo, backendAPI } from '../services/backend-api'
import { useAppStore } from '../store/app'

export const FileManager: React.FC = () => {
  const { files, setFiles, selectedFile, setSelectedFile, setError } = useAppStore()
  const [searchTerm, setSearchTerm] = useState('')
  const [isDragging, setIsDragging] = useState(false)
  const [isLoading, setIsLoading] = useState(true)

  // ファイル一覧を読み込み
  useEffect(() => {
    const loadFiles = async () => {
      try {
        setIsLoading(true)
        const { items } = await backendAPI.getFiles(1, 50)
        setFiles(items)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'ファイル一覧読み込み失敗')
      } finally {
        setIsLoading(false)
      }
    }

    loadFiles()
  }, [setFiles, setError])

  const filteredFiles = files.filter((f) =>
    f.filename.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault()
      setIsDragging(false)

      const droppedFiles = Array.from(e.dataTransfer.files)

      try {
        const uploadPromises = droppedFiles.map((file) => backendAPI.uploadFile(file))
        const uploaded = await Promise.all(uploadPromises)
        setFiles([...files, ...uploaded])
      } catch (err) {
        setError(err instanceof Error ? err.message : 'アップロード失敗')
      }
    },
    [files, setFiles, setError]
  )

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback(() => {
    setIsDragging(false)
  }, [])

  const handleDeleteFile = async (fileId: string) => {
    try {
      await backendAPI.deleteFile(fileId)
      setFiles(files.filter((f) => f.id !== fileId))
      if (selectedFile?.id === fileId) {
        setSelectedFile(null)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '削除失敗')
    }
  }

  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString('ja-JP')
  }

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 rounded-l-lg">
      {/* 検索バー */}
      <div className="p-3 border-b border-gray-200 dark:border-gray-700">
        <input
          type="text"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          placeholder="ファイル検索..."
          className="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600
                     bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white
                     focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {/* ドラッグ&ドロップエリア */}
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        className={`flex-1 overflow-y-auto p-2 transition-colors ${
          isDragging ? 'bg-blue-50 dark:bg-blue-900/20 border-2 border-dashed border-blue-400' : ''
        }`}
      >
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-gray-500 dark:text-gray-400">読み込み中...</div>
          </div>
        ) : filteredFiles.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400">
            <svg className="w-12 h-12 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            <p className="text-sm">ファイルをドラッグ&ドロップ</p>
          </div>
        ) : (
          <ul className="space-y-1">
            {filteredFiles.map((file) => (
              <li
                key={file.id}
                className={`flex items-center px-3 py-2 rounded-lg cursor-pointer transition-colors group ${
                  selectedFile?.id === file.id
                    ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'
                }`}
                onClick={() => setSelectedFile(file)}
              >
                <svg className="w-5 h-5 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1.5}
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{file.filename}</p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {formatSize(file.size)} • {formatDate(file.uploaded_at)}
                  </p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    handleDeleteFile(file.id)
                  }}
                  className="ml-2 p-1 opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 rounded transition-all"
                  title="削除"
                >
                  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                    <path
                      fillRule="evenodd"
                      d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                      clipRule="evenodd"
                    />
                  </svg>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* ステータスバー */}
      <div className="px-3 py-2 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700">
        {filteredFiles.length} ファイル
      </div>
    </div>
  )
}
