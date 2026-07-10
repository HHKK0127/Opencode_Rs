import { Show } from 'solid-js';
import { authState } from '../store/auth';
import { api } from '../services/api';
import { uiState, uiActions } from '../store/ui';
import FileExplorer from '../components/FileExplorer';
import type { FileItem } from '../types/api';

interface DashboardPageProps {
  onNavigate: (page: string, params?: Record<string, string>) => void;
}

function DashboardPage(props: DashboardPageProps) {
  const handleLogout = async () => {
    await api.auth.logout();
  };

  const handleFileSelect = (file: FileItem) => {
    props.onNavigate('editor', { id: file.id, filename: file.filename });
  };

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
        padding: '0.75rem 1.5rem',
        'border-bottom': '1px solid rgba(255,255,255,0.06)',
        '-webkit-app-region': 'drag'
      }}>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '0.75rem' }}>
          <h1 style={{ 'font-size': '1.2rem', color: '#e0e0ff', margin: 0, 'font-weight': 600 }}>
            OpenCode
          </h1>
          <span style={{ 'font-size': '0.75rem', color: '#555577', 'background-color': 'rgba(255,255,255,0.04)', padding: '0.15rem 0.5rem', 'border-radius': '4px' }}>
            {uiState.activePanel}
          </span>
        </div>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '1rem', '-webkit-app-region': 'no-drag' }}>
          <Show when={authState.user}>
            <span style={{ 'font-size': '0.8rem', color: '#8888aa' }}>
              {authState.user!.username}
            </span>
          </Show>
          <button
            onClick={handleLogout}
            style={{
              padding: '0.35rem 0.75rem',
              'background-color': 'rgba(255, 70, 70, 0.12)',
              border: '1px solid rgba(255, 70, 70, 0.25)',
              'border-radius': '6px',
              color: '#ff6b6b',
              'font-size': '0.78rem',
              cursor: 'pointer',
              transition: 'background-color 0.15s'
            }}
            onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'rgba(255, 70, 70, 0.2)'}
            onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'rgba(255, 70, 70, 0.12)'}
          >
            Sign Out
          </button>
        </div>
      </div>

      {/* Sidebar + Content */}
      <div style={{
        flex: 1,
        display: 'flex',
        overflow: 'hidden'
      }}>
        {/* Sidebar */}
        <Show when={uiState.sidebarOpen}>
          <div style={{
            width: '200px',
            'border-right': '1px solid rgba(255,255,255,0.06)',
            'background-color': 'rgba(0,0,0,0.15)',
            display: 'flex',
            'flex-direction': 'column',
            padding: '0.5rem 0'
          }}>
            <button
              onClick={() => uiActions.setActivePanel('files')}
              style={{
                padding: '0.6rem 1rem',
                background: uiState.activePanel === 'files' ? 'rgba(85, 85, 204, 0.2)' : 'transparent',
                border: 'none',
                color: uiState.activePanel === 'files' ? '#e0e0ff' : '#8888aa',
                'font-size': '0.85rem',
                cursor: 'pointer',
                'text-align': 'left',
                'border-left': uiState.activePanel === 'files' ? '3px solid #5555cc' : '3px solid transparent'
              }}
            >
              📁 Files
            </button>
            <button
              onClick={() => uiActions.setActivePanel('search')}
              style={{
                padding: '0.6rem 1rem',
                background: uiState.activePanel === 'search' ? 'rgba(85, 85, 204, 0.2)' : 'transparent',
                border: 'none',
                color: uiState.activePanel === 'search' ? '#e0e0ff' : '#8888aa',
                'font-size': '0.85rem',
                cursor: 'pointer',
                'text-align': 'left',
                'border-left': uiState.activePanel === 'search' ? '3px solid #5555cc' : '3px solid transparent'
              }}
            >
              🔍 Search
            </button>
            <button
              onClick={() => uiActions.setActivePanel('settings')}
              style={{
                padding: '0.6rem 1rem',
                background: uiState.activePanel === 'settings' ? 'rgba(85, 85, 204, 0.2)' : 'transparent',
                border: 'none',
                color: uiState.activePanel === 'settings' ? '#e0e0ff' : '#8888aa',
                'font-size': '0.85rem',
                cursor: 'pointer',
                'text-align': 'left',
                'border-left': uiState.activePanel === 'settings' ? '3px solid #5555cc' : '3px solid transparent'
              }}
            >
              ⚙️ Settings
            </button>

            <div style={{ flex: 1 }} />

            <button
              onClick={() => uiActions.toggleSidebar()}
              style={{
                padding: '0.5rem 1rem',
                background: 'transparent',
                border: 'none',
                color: '#666688',
                'font-size': '0.8rem',
                cursor: 'pointer',
                'text-align': 'left',
                'border-top': '1px solid rgba(255,255,255,0.05)'
              }}
            >
              ◀ Collapse
            </button>
          </div>
        </Show>

        {/* Main Content */}
        <div style={{
          flex: 1,
          display: 'flex',
          'flex-direction': 'column',
          overflow: 'auto'
        }}>
          <FileExplorer onFileSelect={handleFileSelect} />
        </div>
      </div>
    </div>
  );
}

export default DashboardPage;