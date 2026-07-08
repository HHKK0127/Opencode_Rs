/* @refresh reload */
import { render } from 'solid-js/web';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';
import App from './App';
import { authActions } from './store/auth';

console.log('[Renderer] main.tsx loaded');

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 3,
      refetchOnWindowFocus: false
    }
  }
});

console.log('[Renderer] QueryClient created');

authActions.restoreFromStore().then((restored) => {
  if (restored) {
    console.log('[Auth] Session restored from store');
  }
}).catch((err) => {
  console.error('[Auth] Restore failed:', err);
});

const root = document.getElementById('root');
console.log('[Renderer] Root element:', root);

if (root) {
  try {
    render(() => (
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    ), root);
    console.log('[Renderer] App rendered successfully');
  } catch (err) {
    console.error('[Renderer] Render failed:', err);
  }
} else {
  console.error('[Renderer] Root element not found!');
}