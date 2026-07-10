import { contextBridge, ipcRenderer } from 'electron';

type StoreKey = 'token' | 'refreshToken' | 'user' | 'windowBounds';

const electronAPI = {
  store: {
    get: (key: StoreKey) => ipcRenderer.invoke('store:get', key),
    set: (key: StoreKey, value: unknown) => ipcRenderer.invoke('store:set', key, value),
    delete: (key: StoreKey) => ipcRenderer.invoke('store:delete', key)
  },

  app: {
    getVersion: () => ipcRenderer.invoke('app:get-version'),
    getPlatform: () => ipcRenderer.invoke('app:get-platform')
  },

  platform: process.platform,

  versions: {
    node: process.versions.node,
    electron: process.versions.electron,
    chrome: process.versions.chrome
  },

  onMenuAction: (callback: (channel: string) => void) => {
    const handler = (_event: Electron.IpcRendererEvent, channel: string) => callback(channel);
    ipcRenderer.on('menu:new-file', handler);
    ipcRenderer.on('menu:open-file', handler);
    ipcRenderer.on('menu:save', handler);
    ipcRenderer.on('menu:docs', handler);
    return () => {
      ipcRenderer.removeListener('menu:new-file', handler);
      ipcRenderer.removeListener('menu:open-file', handler);
      ipcRenderer.removeListener('menu:save', handler);
      ipcRenderer.removeListener('menu:docs', handler);
    };
  },

  onKeyboardShortcut: (channel: string, callback: () => void) => {
    const handler = () => callback();
    ipcRenderer.on(channel, handler);
    return () => ipcRenderer.removeListener(channel, handler);
  }
};

contextBridge.exposeInMainWorld('electronAPI', electronAPI);

export type ElectronAPI = typeof electronAPI;

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}