function App() {
  return (
    <div style={{ 
      width: '100%', 
      height: '100%',
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
      'background-color': '#1a1a2e',
      color: '#eee'
    }}>
      <div style={{ 'text-align': 'center' }}>
        <h1 style={{ 'font-size': '2.5rem', 'margin-bottom': '1rem' }}>
          OpenCode
        </h1>
        <p style={{ 'font-size': '1.2rem', opacity: 0.8 }}>
          SolidJS + Electron Desktop App
        </p>
        <p style={{ 'margin-top': '2rem', 'font-size': '0.9rem', opacity: 0.6 }}>
          Phase 0 Complete - Security Hardened Scaffold
        </p>
      </div>
    </div>
  );
}

export default App;