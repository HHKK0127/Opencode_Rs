import React, { useState, useRef } from 'react'
import { initializeFileUpload, uploadFileChunk, completeFileUpload } from '../services/api'
import '../styles/FileUpload.css'

interface FileUploadPanelProps {
  onUploadComplete?: () => void
  onError?: (error: string) => void
}

export const FileUploadPanel: React.FC<FileUploadPanelProps> = ({ onUploadComplete, onError }) => {
  const [file, setFile] = useState<File | null>(null)
  const [isUploading, setIsUploading] = useState(false)
  const [progress, setProgress] = useState(0)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const CHUNK_SIZE = 1048576 // 1MB

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0]
    if (selectedFile) {
      setFile(selectedFile)
      setProgress(0)
    }
  }

  const handleUpload = async () => {
    if (!file) return

    try {
      setIsUploading(true)
      setProgress(0)

      // 1. セッション初期化
      const initRes = await initializeFileUpload({
        file_name: file.name,
        file_size: file.size,
        mime_type: file.type || 'application/octet-stream',
        chunk_size: CHUNK_SIZE,
      })

      const sid = initRes.session_id
      if (!sid) {
        throw new Error('Failed to initialize upload session')
      }

      // 2. チャンクアップロード
      const chunks = Math.ceil(file.size / CHUNK_SIZE)
      for (let i = 0; i < chunks; i++) {
        const start = i * CHUNK_SIZE
        const end = Math.min(start + CHUNK_SIZE, file.size)
        const chunk = file.slice(start, end)

        await uploadFileChunk(sid, i, chunk)

        const percent = Math.round(((i + 1) / chunks) * 100)
        setProgress(percent)
      }

      // 3. 完了処理
      await completeFileUpload(sid)

      setProgress(100)
      setFile(null)
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }

      onUploadComplete?.()
    } catch (error) {
      const message = error instanceof Error ? error.message : 'アップロード失敗'
      onError?.(message)
    } finally {
      setIsUploading(false)
    }
  }

  return (
    <div className="file-upload-panel">
      <div className="upload-area">
        <input
          ref={fileInputRef}
          type="file"
          onChange={handleFileSelect}
          disabled={isUploading}
          className="file-input"
        />

        <div className="upload-info">
          {file ? (
            <>
              <p className="file-name">📄 {file.name}</p>
              <p className="file-size">({(file.size / 1024 / 1024).toFixed(2)} MB)</p>
            </>
          ) : (
            <p className="placeholder">ファイルを選択してください</p>
          )}
        </div>

        {file && (
          <button
            onClick={handleUpload}
            disabled={isUploading}
            className="upload-button"
          >
            {isUploading ? `アップロード中... ${progress}%` : 'アップロード'}
          </button>
        )}

        {progress > 0 && isUploading && (
          <div className="progress-container">
            <div className="progress-bar">
              <div
                className="progress-fill"
                style={{ width: `${progress}%` }}
              />
            </div>
            <span className="progress-text">{progress}%</span>
          </div>
        )}
      </div>
    </div>
  )
}
