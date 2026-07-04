import { useRef, useEffect } from 'react'
import './Composer.css'

interface ComposerProps {
  value: string
  onChange: (value: string) => void
  onSend: () => void
  isLoading?: boolean
}

export const Composer = ({ value, onChange, onSend, isLoading }: ComposerProps) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto'
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`
    }
  }, [value])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (!isLoading) onSend()
    }
  }

  return (
    <div data-component="composer-area">
      <div className="composer-inner">
        <textarea
          ref={textareaRef}
          data-component="text-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a message..."
          rows={1}
          disabled={isLoading}
        />
        <button
          data-component="send-button"
          onClick={onSend}
          disabled={!value.trim() || isLoading}
        >
          {isLoading ? '...' : 'Send'}
        </button>
      </div>
    </div>
  )
}
