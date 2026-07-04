import axios from 'axios'

const API_BASE = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'
const AUTH = 'Basic ' + btoa('opencode:opencode')

export const api = axios.create({
  baseURL: `${API_BASE}/v2`,
  headers: {
    'Authorization': AUTH,
    'Content-Type': 'application/json',
  },
})

export const createSession = async () => {
  const res = await api.post('/session')
  return res.data
}

export const sendPrompt = async (sessionId: string, prompt: string) => {
  const res = await api.post(`/session/${sessionId}/prompt`, { prompt })
  return res.data
}

export const fetchMessages = async (sessionId: string) => {
  const res = await api.get(`/session/${sessionId}/messages`)
  return res.data
}
