import { contextBridge, ipcRenderer } from 'electron';

type StoreKey = 'token' | 'refreshToken' | 'user' | 'windowBounds';

contextBridge.exposeInMainWorld('electronAPI', {
  store: {
    get: (key: StoreKey) => ipcRenderer.invoke('store:get', key),
    set: (key: StoreKey, value: unknown) => ipcRenderer.invoke('store:set', key, value),
    delete: (key: StoreKey) => ipcRenderer.invoke('store:delete', key)
  },
  
  platform: process.platform,
  
  versions: {
    node: process.versions.node,
    electron: process.versions.electron,
    chrome: process.versions.chrome
  }
});

export interface ElectronAPI {
  store: {
    get(key: StoreKey): Promise<unknown>;
    set(key: StoreKey, value: unknown): Promise<void>;
    delete(key: StoreKey): Promise<void>;
  };
  platform: string;
  versions: {
    node: string;
    electron: string;
    chrome: string;
  };
}

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}