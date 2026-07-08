import { Show } from 'solid-js';
import { authState } from './store/auth';
import { api } from './services/api';
import LoginForm from './components/LoginForm';

function Dashboard() {
  return (
    <div style={{
      width: '100%',
      height: '100%',
      display: 'flex',
      'flex-direction': 'column',
      'background-color': '#1a1a2e',
      color: '#eee'
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'space-between',
        padding: '1rem 1.5rem',
        'border-bottom': '1px solid rgba(255,255,255,0.06)',
        '-webkit-app-region': 'drag'
      }}>
        <h1 style={{ 'font-size': '1.3rem', color: '#e0e0ff', margin: 0 }}>
          OpenCode
        </h1>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '1rem', '-webkit-app-region': 'no-drag' }}>
          <span style={{ 'font-size': '0.8rem', color: '#8888aa' }}>
            {authState.user?.username}
          </span>
          <button
            onClick={() => api.auth.logout()}
            style={{
              padding: '0.4rem 0.8rem',
              'background-color': 'rgba(255, 70, 70, 0.15)',
              border: '1px solid rgba(255, 70, 70, 0.3)',
              'border-radius': '6px',
              color: '#ff6b6b',
              'font-size': '0.8rem',
              cursor: 'pointer'
            }}
          >
            Sign Out
          </button>
        </div>
      </div>

      {/* Content */}
      <div style={{
        flex: 1,
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'center',
        padding: '2rem'
      }}>
        <div style={{ 'text-align': 'center' }}>
          <p style={{ 'font-size': '1.1rem', opacity: 0.7 }}>
            Welcome to OpenCode Desktop
          </p>
          <p style={{ 'margin-top': '1rem', 'font-size': '0.85rem', opacity: 0.4 }}>
            Phase 2 Complete — Authentication UI
          </p>
        </div>
      </div>
    </div>
  );
}

function App() {
  return (
    <Show when={authState.isAuthenticated} fallback={<LoginForm />}>
      <Dashboard />
    </Show>
  );
}

export default App;