import React, { useEffect, useState } from 'react'
import { listFiles, downloadFile } from '../services/api'
import '../styles/FileList.css'

interface File {
  id: string
  filename: string
  size: number
  mime_type: string
  uploaded_at: string
}

interface FileListProps {
  refresh?: number
  onError?: (error: string) => void
}

export const FileList: React.FC<FileListProps> = ({ refresh = 0, onError }) => {
  const [files, setFiles] = useState<File[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadFiles()
  }, [refresh])

  const loadFiles = async () => {
    try {
      setLoading(true)
      setError(null)
      const data = await listFiles()
      setFiles(data || [])
    } catch (err) {
      const message = err instanceof Error ? err.message : 'ファイル一覧取得失敗'
      setError(message)
      onError?.(message)
    } finally {
      setLoading(false)
    }
  }

  const handleDownload = (fileId: string, filename: string) => {
    const url = downloadFile(fileId)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    link.click()
  }

  const formatFileSize = (bytes: number) => {
    const units = ['B', 'KB', 'MB', 'GB']
    let size = bytes
    let unitIndex = 0
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024
      unitIndex++
    }
    return `${size.toFixed(2)} ${units[unitIndex]}`
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  if (loading) {
    return <div className="file-list-loading">読み込み中...</div>
  }

  if (error) {
    return <div className="file-list-error">❌ {error}</div>
  }

  if (files.length === 0) {
    return <div className="file-list-empty">📂 ファイルがありません</div>
  }

  return (
    <div className="file-list">
      <h3 className="file-list-title">📁 ファイル一覧</h3>
      <div className="file-table">
        <div className="file-header">
          <div className="file-name-col">ファイル名</div>
          <div className="file-size-col">サイズ</div>
          <div className="file-date-col">日時</div>
          <div className="file-action-col">操作</div>
        </div>

        {files.map((file) => (
          <div key={file.id} className="file-row">
            <div className="file-name-col">
              <span className="file-icon">📄</span>
              {file.filename}
            </div>
            <div className="file-size-col">{formatFileSize(file.size)}</div>
            <div className="file-date-col">{formatDate(file.uploaded_at)}</div>
            <div className="file-action-col">
              <button
                className="download-btn"
                onClick={() => handleDownload(file.id, file.filename)}
              >
                ⬇️ DL
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
