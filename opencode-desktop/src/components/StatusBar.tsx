import React from 'react'
import { useAppStore } from '../store/app'

export const StatusBar: React.FC = () => {
  const { selectedFile, user, isDarkMode } = useAppStore()

  return (
    <div
      className="flex items-center justify-between px-4 py-1.5 
                     bg-gray-100 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700
                     text-xs text-gray-600 dark:text-gray-400 rounded-b-lg"
    >
      <div className="flex items-center space-x-4">
        {selectedFile ? (
          <>
            <span className="font-medium">{selectedFile.filename}</span>
            <span>{selectedFile.size.toLocaleString()} bytes</span>
          </>
        ) : (
          <span>ファイル未選択</span>
        )}
      </div>

      <div className="flex items-center space-x-4">
        {user && (
          <span className="flex items-center">
            <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
                clipRule="evenodd"
              />
            </svg>
            {user.username}
          </span>
        )}
        <span className={`w-2 h-2 rounded-full ${isDarkMode ? 'bg-yellow-400' : 'bg-blue-400'}`} />
      </div>
    </div>
  )
}
