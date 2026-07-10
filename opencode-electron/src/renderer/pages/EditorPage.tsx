interface EditorPageProps {
  params?: Record<string, string>;
  onNavigate: (page: string, params?: Record<string, string>) => void;
}

function EditorPage(props: EditorPageProps) {
  const filename = props.params?.filename || 'Untitled';

  return (
    <div style={{
      width: '100%',
      height: '100%',
      display: 'flex',
      'flex-direction': 'column',
      'background-color': '#1a1a2e',
      color: '#eee'
    }}>
      {/* Editor Header */}
      <div style={{
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'space-between',
        padding: '0.5rem 1rem',
        'border-bottom': '1px solid rgba(255,255,255,0.06)',
        '-webkit-app-region': 'drag',
        'background-color': 'rgba(0,0,0,0.15)'
      }}>
        <div style={{ display: 'flex', 'align-items': 'center', gap: '0.75rem', '-webkit-app-region': 'no-drag' }}>
          <button
            onClick={() => props.onNavigate('dashboard')}
            style={{
              padding: '0.25rem 0.6rem',
              'background-color': 'rgba(255,255,255,0.05)',
              border: '1px solid rgba(255,255,255,0.1)',
              'border-radius': '4px',
              color: '#8888aa',
              cursor: 'pointer',
              'font-size': '0.78rem'
            }}
            onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.1)'}
            onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.05)'}
          >
            ← Back
          </button>
          <span style={{ 'font-size': '0.9rem', color: '#ccccee' }}>{filename}</span>
        </div>
        <div style={{ 'font-size': '0.75rem', color: '#555577' }}>
          Read-only
        </div>
      </div>

      {/* Editor Content Area */}
      <div style={{
        flex: 1,
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'center',
        padding: '2rem'
      }}>
        <div style={{ 'text-align': 'center' }}>
          <p style={{ 'font-size': '1.2rem', opacity: 0.5, 'margin-bottom': '0.5rem' }}>📝</p>
          <p style={{ 'font-size': '0.9rem', opacity: 0.4 }}>
            File content will be displayed here
          </p>
          <p style={{ 'font-size': '0.8rem', opacity: 0.25, 'margin-top': '0.5rem' }}>
            Phase 4: Monaco Editor integration
          </p>
        </div>
      </div>
    </div>
  );
}

export default EditorPage;