import { authState, authActions } from '../store/auth';
import type { 
  ApiResponse,
  LoginData,
  FilesData
} from '../types/api';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api/v1';
const REQUEST_TIMEOUT = 10000;

const validateEnvelope = <T>(json: unknown): ApiResponse<T> => {
  if (!json || typeof json !== 'object') {
    throw new Error('Invalid response format: not an object');
  }

  const response = json as Record<string, unknown>;

  if ('code' in response && 'error' in response) {
    const errorMsg = String(response.error);
    const code = Number(response.code) || 500;
    throw new Error(`API Error [${code}]: ${errorMsg}`);
  }

  if (!('status' in response)) {
    throw new Error('Invalid response format: missing status field');
  }

  const envelope = json as ApiResponse<T>;

  if (envelope.status === 'error') {
    throw new Error(envelope.error || 'Unknown API error');
  }

  return envelope;
};

const extractData = <T>(response: ApiResponse<T>): T => {
  if (response.status === 'error') {
    throw new Error(response.error || 'Unknown API error');
  }
  if (response.data === null || response.data === undefined) {
    throw new Error('Empty data in success response');
  }
  return response.data;
};

const fetchWithTimeout = async (
  url: string,
  options: RequestInit,
  timeout = REQUEST_TIMEOUT
): Promise<Response> => {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);
    if (error instanceof Error && error.name === 'AbortError') {
      throw new Error('Request timeout after ' + timeout + 'ms');
    }
    throw error;
  }
};

const callApi = async <T>(
  url: string,
  options: RequestInit = {},
  retryCount = 0
): Promise<T> => {
  const token = authState.token;

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {})
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetchWithTimeout(url, {
    ...options,
    headers
  });

  if (response.status === 401 && retryCount === 0) {
    const refreshed = await refreshToken();
    if (refreshed) {
      return callApi(url, options, retryCount + 1);
    }
    authActions.logout();
    window.location.href = '/';
    throw new Error('Session expired. Please login again.');
  }

  if (!response.ok) {
    let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
    try {
      const errorJson = await response.json();
      if (errorJson && typeof errorJson === 'object') {
        if ('error' in errorJson) {
          errorMessage = String(errorJson.error);
        } else if ('message' in errorJson) {
          errorMessage = String(errorJson.message);
        }
      }
    } catch {
      // JSONパース失敗 → デフォルトメッセージを使用
    }
    throw new Error(errorMessage);
  }

  const json = await response.json();
  const envelope = validateEnvelope<T>(json);
  return extractData(envelope);
};

const refreshToken = async (): Promise<boolean> => {
  const refreshToken = authState.refreshToken;
  if (!refreshToken) return false;

  try {
    const response = await fetchWithTimeout(
      `${API_BASE_URL}/auth/refresh`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: refreshToken })
      }
    );

    if (!response.ok) return false;

    const json = await response.json();
    const envelope = validateEnvelope<{ token: string }>(json);
    const data = extractData(envelope);

    authActions.setToken(data.token);
    await window.electronAPI.store.set('token', data.token);
    return true;
  } catch (error) {
    console.error('[Auth] Token refresh failed:', error);
    return false;
  }
};

export const api = {
  auth: {
    login: async (username: string, password: string) => {
      try {
        authActions.setLoading(true);
        authActions.setError(null);

        const data = await callApi<LoginData>(`${API_BASE_URL}/auth/login`, {
          method: 'POST',
          body: JSON.stringify({ username, password })
        });

        authActions.setToken(data.token);
        authActions.setRefreshToken(data.refresh_token);
        authActions.setUser(data.user);

        await window.electronAPI.store.set('token', data.token);
        await window.electronAPI.store.set('refreshToken', data.refresh_token);
        await window.electronAPI.store.set('user', data.user);

        return { data, error: null };
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Login failed';
        authActions.setError(message);
        return { data: null, error: message };
      } finally {
        authActions.setLoading(false);
      }
    },

    logout: async () => {
      authActions.logout();
      await window.electronAPI.store.delete('token');
      await window.electronAPI.store.delete('refreshToken');
      await window.electronAPI.store.delete('user');
    }
  },

  files: {
    list: async (page = 1, perPage = 20) => {
      try {
        const data = await callApi<FilesData>(
          `${API_BASE_URL}/files?page=${page}&per_page=${perPage}`
        );
        return { data, error: null };
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to fetch files';
        return { data: null, error: message };
      }
    },

    get: async (id: string) => {
      try {
        const data = await callApi<string>(
          `${API_BASE_URL}/files/${id}/download`
        );
        return { data, error: null };
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to fetch file';
        return { data: null, error: message };
      }
    }
  }
};