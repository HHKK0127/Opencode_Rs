import { useState, useEffect, useRef } from 'react'
import { Sidebar } from './components/Sidebar'
import { ChatContainer } from './components/ChatContainer'
import { Composer } from './components/Composer'
import { ErrorToast } from './components/ErrorToast'
import { useMessages } from './services/useMessages'
import './styles/app.css'

const App = () => {
  const { messages, sendMessage, isLoading, error, setError } = useMessages()
  const [inputValue, setInputValue] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSend = () => {
    if (!inputValue.trim()) return
    sendMessage(inputValue)
    setInputValue('')
  }

  return (
    <div className="app-container">
      <Sidebar />
      <div className="main-area">
        <div className="session-header">
          <h2>New Chat</h2>
        </div>
        <ChatContainer messages={messages} messagesEndRef={messagesEndRef} isLoading={isLoading} />
        <Composer
          value={inputValue}
          onChange={setInputValue}
          onSend={handleSend}
          isLoading={isLoading}
        />
      </div>
      {error && <ErrorToast message={error} onClose={() => setError(null)} />}
    </div>
  )
}

export default App
