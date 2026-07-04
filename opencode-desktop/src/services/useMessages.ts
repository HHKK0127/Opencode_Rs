import { useState, useCallback } from 'react'
import { Message } from '../types'
import { createSession, sendPrompt } from './api'

export const useMessages = () => {
  const [messages, setMessages] = useState<Message[]>([])
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const initSession = useCallback(async () => {
    try {
      const session = await createSession()
      setSessionId(session.id)
      setError(null)
      return session.id
    } catch (_err) {
      const message = 'Failed to create session. Please try again.'
      console.error('Failed to create session:', _err)
      setError(message)
      return null
    }
  }, [])

  const sendMessage = useCallback(async (content: string) => {
    if (!content.trim()) return

    setError(null)

    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: content.trim(),
      timestamp: new Date().toISOString(),
    }

    setMessages((prev) => [...prev, userMsg])
    setIsLoading(true)

    try {
      let sid = sessionId
      if (!sid) {
        sid = await initSession()
        if (!sid) throw new Error('Session creation failed')
      }

      await sendPrompt(sid, content.trim())

      // Stage 1: Simulation (replace with SSE in Stage 2)
      setTimeout(() => {
        const assistantMsg: Message = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: 'Received your message. SSE streaming will be implemented in Stage 2.',
          timestamp: new Date().toISOString(),
        }
        setMessages((prev) => [...prev, assistantMsg])
        setIsLoading(false)
      }, 1000)
    } catch (_err) {
      const message = 'Failed to send message. Please try again.'
      console.error('Failed to send message:', _err)
      setError(message)
      setIsLoading(false)
    }
  }, [sessionId, initSession])

  return { messages, sendMessage, isLoading, error, setError, sessionId }
}
