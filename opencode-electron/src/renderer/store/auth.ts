import { createStore } from 'solid-js/store';

interface User {
  id: string;
  username: string;
}

interface AuthState {
  token: string | null;
  refreshToken: string | null;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

const [authState, setAuthState] = createStore<AuthState>({
  token: null,
  refreshToken: null,
  user: null,
  isAuthenticated: false,
  isLoading: false,
  error: null
});

export const authActions = {
  setToken: (token: string): void => {
    setAuthState({ token, isAuthenticated: true });
  },
  
  setRefreshToken: (refreshToken: string): void => {
    setAuthState({ refreshToken });
  },
  
  setUser: (user: User): void => {
    setAuthState({ user });
  },
  
  setLoading: (isLoading: boolean): void => {
    setAuthState({ isLoading });
  },
  
  setError: (error: string | null): void => {
    setAuthState({ error });
  },
  
  logout: (): void => {
    setAuthState({
      token: null,
      refreshToken: null,
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null
    });
  },
  
  restoreFromStore: async (): Promise<boolean> => {
    try {
      const token = await window.electronAPI.store.get('token') as string;
      const refreshToken = await window.electronAPI.store.get('refreshToken') as string;
      const user = await window.electronAPI.store.get('user') as User | null;
      
      if (token && user) {
        setAuthState({
          token,
          refreshToken,
          user,
          isAuthenticated: true
        });
        return true;
      }
      return false;
    } catch (error) {
      console.error('Failed to restore auth from store:', error);
      return false;
    }
  }
};

export { authState };