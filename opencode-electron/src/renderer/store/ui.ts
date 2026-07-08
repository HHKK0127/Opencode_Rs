import { createStore } from 'solid-js/store';

interface Notification {
  id: string;
  type: 'success' | 'error' | 'info';
  message: string;
}

interface UIState {
  sidebarOpen: boolean;
  darkMode: boolean;
  activePanel: 'files' | 'search' | 'settings';
  notifications: Notification[];
}

const [uiState, setUIState] = createStore<UIState>({
  sidebarOpen: true,
  darkMode: true,
  activePanel: 'files',
  notifications: []
});

export const uiActions = {
  toggleSidebar: (): void => {
    setUIState('sidebarOpen', (v) => !v);
  },
  
  toggleDarkMode: (): void => {
    setUIState('darkMode', (v) => !v);
  },
  
  setActivePanel: (panel: 'files' | 'search' | 'settings'): void => {
    setUIState({ activePanel: panel });
  },
  
  addNotification: (type: 'success' | 'error' | 'info', message: string): void => {
    const id = Date.now().toString();
    setUIState('notifications', (prev) => [...prev, { id, type, message }]);
    
    setTimeout(() => {
      setUIState('notifications', (prev) => prev.filter((n) => n.id !== id));
    }, 5000);
  },
  
  removeNotification: (id: string): void => {
    setUIState('notifications', (prev) => prev.filter((n) => n.id !== id));
  }
};

export { uiState };