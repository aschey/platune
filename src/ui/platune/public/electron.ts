import { ipcMain, app, BrowserWindow, protocol } from 'electron';
import path from 'path';
import isDev from 'electron-is-dev';
import { spawn } from 'child_process';
import net from 'net';
import contextMenu from 'electron-context-menu';

let mainWindow: BrowserWindow | null;
let server: net.Server | null;

const createWindow = () => {
  const dispose = contextMenu();
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    frame: false,
    backgroundColor: '#000',
    icon: getIcon(),
    webPreferences: {
      webSecurity: !isDev,
      nodeIntegration: true,
      nodeIntegrationInWorker: false,
      backgroundThrottling: false,
    },
  });
  mainWindow.loadURL(isDev ? 'http://localhost:3000' : `file://${path.join(__dirname, '../build/index.html')}`);

  mainWindow.on('closed', () => {
    mainWindow = null;
    server?.close();
    server = null;
  });

  ipcMain.handle('close', async () => mainWindow?.close());

  ipcMain.handle('restoreMax', async () => {
    if (mainWindow?.isMaximized()) {
      mainWindow?.restore();
    } else {
      mainWindow?.maximize();
    }
  });

  ipcMain.handle('minimize', async () => mainWindow?.minimize());
};

app.on('ready', async () => {
  if (isDev) {
    protocol.registerFileProtocol('file', (request, cb) => {
      const pathname = decodeURI(request.url.replace('file:///', ''));
      cb(pathname);
    });
    await installExtensions();
  }

  const spawnServer = !isDev;
  if (spawnServer) {
    server = net.createServer();
    server.listen(8001);
    let command =
      process.platform === 'win32'
        ? `${app.getPath('home')}\\AppData\\Local\\Programs\\platune-server\\platune-server.exe`
        : '/opt/platune/platune-server';
    let proc = spawn(command, { detached: true, windowsHide: true, shell: false, stdio: 'ignore' });
    proc.unref();
  }

  createWindow();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});

const getIcon = () => {
  if (process.platform === 'linux') {
    return path.join(__dirname, '../public/res/icon.png');
  } else if (isDev) {
    return path.join(__dirname, '../public/res/favicon.ico');
  } else {
    return undefined;
  }
};

const installExtensions = async () => {
  const { default: installExtension, REACT_DEVELOPER_TOOLS, REDUX_DEVTOOLS } = require('electron-devtools-installer');
  const forceDownload = true;
  await installExtension(REACT_DEVELOPER_TOOLS, forceDownload);
  await installExtension(REDUX_DEVTOOLS, forceDownload);
};
