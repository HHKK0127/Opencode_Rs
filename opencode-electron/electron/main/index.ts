import { app, BrowserWindow, ipcMain, dialog, Menu, globalShortcut } from 'electron';
import path from 'path';
import log from 'electron-log';
import Store from 'electron-store';
import 'dotenv/config';
import { createAppMenu, registerMenuIpc } from './menu';

const IS_PRODUCTION = process.env.NODE_ENV === 'production';
const ENCRYPTION_KEY = process.env.OPENCODE_ENCRYPTION_KEY;

interface StoreSchema {
  token: string;
  refreshToken: string;
  user: {
    id: string;
    username: string;
  } | null;
  windowBounds: {
    width: number;
    height: number;
    x?: number;
    y?: number;
  };
}

let store: Store<StoreSchema>;

const initializeStore = (): void => {
  if (IS_PRODUCTION && !ENCRYPTION_KEY) {
    log.error('OPENCODE_ENCRYPTION_KEY is required in production');
    dialog.showErrorBox(
      'Configuration Error',
      'OPENCODE_ENCRYPTION_KEY environment variable is not set.\nPlease set a secure encryption key and restart the application.'
    );
    app.quit();
    process.exit(1);
    return;
  }

  store = new Store<StoreSchema>({
    defaults: {
      token: '',
      refreshToken: '',
      user: null,
      windowBounds: { width: 1200, height: 800 }
    },
    encryptionKey: ENCRYPTION_KEY || 'dev-key-only-never-use-in-production',
    clearInvalidConfig: true
  });
};

log.info('OpenCode Electron main process starting...');

let mainWindow: BrowserWindow | null = null;

const createWindow = (): void => {
  const bounds = store.get('windowBounds');
  
  mainWindow = new BrowserWindow({
    width: bounds.width,
    height: bounds.height,
    x: bounds.x,
    y: bounds.y,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      preload: path.join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      allowRunningInsecureContent: false,
      experimentalFeatures: false
    },
    titleBarStyle: 'hiddenInset',
    show: true  // デバッグ用: 即座に表示
  });

  if (process.env.VITE_DEV_SERVER_URL) {
    console.log('[Main] Loading dev server:', process.env.VITE_DEV_SERVER_URL);
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
    if (!IS_PRODUCTION) {
      mainWindow.webContents.openDevTools();
    }
  } else {
    console.log('[Main] Loading index.html from dist');
    // 本番ビルド時は dist/index.html、相対パスで解決
    const indexPath = IS_PRODUCTION
      ? path.join(__dirname, '../../dist/index.html')
      : path.join(__dirname, '../../src/renderer/index.html');
    console.log('[Main] Index path:', indexPath);
    mainWindow.loadFile(indexPath);
  }

  // エラーログ追加
  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription) => {
    console.error('[Main] Failed to load:', errorCode, errorDescription);
  });

  mainWindow.webContents.on('render-process-gone', (_event, details) => {
    console.error('[Main] Renderer process gone:', details.reason);
  });

  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
  });

  mainWindow.on('close', () => {
    if (mainWindow) {
      store.set('windowBounds', mainWindow.getBounds());
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
};

type StoreKey = 'token' | 'refreshToken' | 'user' | 'windowBounds';

ipcMain.handle('store:get', async (_, key: StoreKey) => {
  return store.get(key);
});

ipcMain.handle('store:set', async (_, key: StoreKey, value: unknown) => {
  store.set(key, value);
});

ipcMain.handle('store:delete', async (_, key: StoreKey) => {
  store.delete(key);
});

process.on('uncaughtException', (error) => {
  log.error('Uncaught exception:', error);
  dialog.showErrorBox('Error', `An unexpected error occurred:\n${error.message}`);
});

app.whenReady().then(() => {
  initializeStore();
  registerMenuIpc();
  createWindow();

  const menu = createAppMenu();
  Menu.setApplicationMenu(menu);

  // ショートカット登録: メニューの accelerator だけで動作するが、
  // globalShortcut はウィンドウにフォーカスがない時も機能する
  app.on('browser-window-focus', () => {
    // Electron 標準の accelerator で十分なので globalShortcut は未使用
    // (将来的に system-wide ショートカットが必要ならここに追加)
  });

  app.on('browser-window-blur', () => {
    globalShortcut.unregisterAll();
  });

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});