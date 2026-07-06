import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import './styles/theme.css'
import './styles/global.css'

// Initialize theme from localStorage
const savedTheme = localStorage.getItem('opencode-color-scheme') || 'dark'
document.documentElement.setAttribute('data-color-scheme', savedTheme)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
