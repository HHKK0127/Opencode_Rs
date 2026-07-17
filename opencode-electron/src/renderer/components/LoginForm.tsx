import { Show, createSignal } from 'solid-js';
import { authState, authActions } from '../store/auth';
import { api } from '../services/api';

const testBypassEnabled = import.meta.env.DEV || import.meta.env.VITE_ALLOW_TEST_LOGIN_BYPASS === 'true';

function LoginForm() {
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [showPassword, setShowPassword] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const trimmedUser = username().trim();
    if (!trimmedUser || !password()) return;
    await api.auth.login(trimmedUser, password());
  };

  const handleRegisterClick = async () => {
    const trimmedUser = username().trim();
    const pw = password();
    if (!trimmedUser || !pw) return;
    try {
      const res = await fetch(
        `${import.meta.env.VITE_API_URL || 'http://localhost:8080/api/v1'}/auth/register`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username: trimmedUser, password: pw })
        }
      );
      if (res.ok) {
        await api.auth.login(trimmedUser, pw);
      } else {
        const err = await res.json().catch(() => ({ error: 'Registration failed' }));
        authActions.setError(err.error || 'Registration failed');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Connection failed';
      authActions.setError(message);
    }
  };

  const handleTestBypass = async () => {
    authActions.loginForTesting();
    await window.electronAPI.store.set('token', 'test-bypass-token');
    await window.electronAPI.store.delete('refreshToken');
    await window.electronAPI.store.set('user', {
      id: 'test-user',
      username: 'testuser'
    });
  };

  return (
    <div style={{
      width: '100%',
      height: '100%',
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
      'background': 'linear-gradient(135deg, #0f0f1e 0%, #1a1a3e 50%, #0f0f1e 100%)'
    }}>
      <div style={{
        width: '360px',
        padding: '2.5rem',
        'background-color': 'rgba(26, 26, 46, 0.95)',
        'border-radius': '12px',
        border: '1px solid rgba(255, 255, 255, 0.08)',
        'box-shadow': '0 8px 32px rgba(0, 0, 0, 0.4)'
      }}>
        <div style={{ 'text-align': 'center', 'margin-bottom': '2rem' }}>
          <h1 style={{
            'font-size': '1.8rem', color: '#e0e0ff', 'margin-bottom': '0.5rem',
            'font-weight': '600', 'letter-spacing': '0.5px'
          }}>
            OpenCode
          </h1>
          <p style={{ 'font-size': '0.85rem', color: '#8888aa', margin: 0 }}>
            Sign in to your workspace
          </p>
        </div>

        <Show when={authState.error}>
          <div style={{
            padding: '0.75rem 1rem',
            'background-color': 'rgba(255, 70, 70, 0.12)',
            border: '1px solid rgba(255, 70, 70, 0.3)',
            'border-radius': '8px', color: '#ff6b6b',
            'font-size': '0.85rem', 'margin-bottom': '1rem', 'text-align': 'center'
          }}>
            {authState.error}
          </div>
        </Show>

        <form onSubmit={handleSubmit}>
          <div style={{ 'margin-bottom': '1.25rem' }}>
            <label style={{
              display: 'block', 'font-size': '0.8rem', color: '#aaaacc',
              'margin-bottom': '0.4rem', 'font-weight': '500'
            }}>
              Username
            </label>
            <input
              type="text"
              value={username()}
              onInput={(e) => setUsername(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmit(e)}
              placeholder="Enter your username"
              disabled={authState.isLoading}
              style={{
                width: '100%', padding: '0.75rem 1rem',
                'background-color': 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                'border-radius': '8px', color: '#e0e0ff',
                'font-size': '0.95rem', outline: 'none',
                'box-sizing': 'border-box'
              }}
            />
          </div>

          <div style={{ 'margin-bottom': '1.5rem' }}>
            <label style={{
              display: 'block', 'font-size': '0.8rem', color: '#aaaacc',
              'margin-bottom': '0.4rem', 'font-weight': '500'
            }}>
              Password
            </label>
            <div style={{ position: 'relative' }}>
              <input
                type={showPassword() ? 'text' : 'password'}
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSubmit(e)}
                placeholder="Enter your password"
                disabled={authState.isLoading}
                style={{
                  width: '100%', padding: '0.75rem 1rem', 'padding-right': '3rem',
                  'background-color': 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  'border-radius': '8px', color: '#e0e0ff',
                  'font-size': '0.95rem', outline: 'none',
                  'box-sizing': 'border-box'
                }}
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword())}
                style={{
                  position: 'absolute', right: '0.75rem', top: '50%',
                  transform: 'translateY(-50%)', background: 'none',
                  border: 'none', color: '#8888aa', cursor: 'pointer',
                  'font-size': '0.8rem', padding: '0.25rem'
                }}
              >
                {showPassword() ? 'Hide' : 'Show'}
              </button>
            </div>
          </div>

          <button
            type="submit"
            disabled={authState.isLoading || !username().trim() || !password()}
            style={{
              width: '100%', padding: '0.8rem',
              'background-color': authState.isLoading || !username().trim() || !password()
                ? 'rgba(100, 100, 200, 0.3)' : '#5555cc',
              color: '#fff', border: 'none', 'border-radius': '8px',
              'font-size': '1rem', 'font-weight': '600',
              cursor: authState.isLoading ? 'wait' : 'pointer',
              'margin-bottom': '0.75rem'
            }}
          >
            {authState.isLoading ? 'Signing in...' : 'Sign In'}
          </button>

          <button
            type="button"
            onClick={handleRegisterClick}
            disabled={authState.isLoading}
            style={{
              width: '100%', padding: '0.8rem',
              background: 'none',
              border: '1px solid rgba(255, 255, 255, 0.12)',
              'border-radius': '8px', color: '#8888bb',
              'font-size': '0.85rem', cursor: 'pointer'
            }}
          >
            Create new account
          </button>

          <Show when={testBypassEnabled}>
            <button
              type="button"
              onClick={handleTestBypass}
              disabled={authState.isLoading}
              style={{
                width: '100%', padding: '0.8rem',
                background: 'none',
                border: '1px dashed rgba(120, 200, 255, 0.45)',
                'border-radius': '8px', color: '#8fd8ff',
                'font-size': '0.85rem', cursor: 'pointer',
                'margin-top': '0.75rem'
              }}
            >
              Skip login (test mode)
            </button>
          </Show>
        </form>

        <div style={{
          'margin-top': '1.5rem', 'text-align': 'center',
          'font-size': '0.75rem', color: '#666688'
        }}>
          <p style={{ margin: 0 }}>Connected to OpenCode_Rs API</p>
        </div>
      </div>
    </div>
  );
}

export default LoginForm;