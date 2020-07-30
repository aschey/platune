import { ipcMain, app, BrowserWindow } from 'electron';
import path from 'path';
import isDev from 'electron-is-dev';
import { spawn } from 'child_process';
import zmq from 'zeromq';
import net from 'net';
import contextMenu from 'electron-context-menu';

let mainWindow: BrowserWindow | null;
let server: net.Server | null;

function createWindow() {
    const dispose = contextMenu();
    mainWindow = new BrowserWindow({width: 900, height: 680, frame: false, backgroundColor: '#000', icon: path.join(__dirname, '../src/res/logo.png'), webPreferences: { 
        webSecurity: !isDev, 
        nodeIntegration: true, 
        nodeIntegrationInWorker: false,
        backgroundThrottling: false
    }});
    mainWindow.loadURL(isDev ? 'http://localhost:3000' : `file://${path.join(__dirname, '../build/index.html')}`);
    
    mainWindow.on('closed', () => {
      mainWindow = null;
      server?.close();
      server = null;
    });
}

app.on('ready', async () => {
  if (isDev) {
    await installExtensions();
  }
  
  const spawnServer = false;
  if (spawnServer) {
    server = net.createServer();
    server.listen(8001);
    let command = process.platform === 'win32' ? '.\\target\\debug\\namp.exe' : './target/debug/namp';
    let proc = spawn(command, {cwd: '../../..', detached: true, windowsHide: true, shell: isDev, stdio: 'ignore'});
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

const installExtensions = async () => {
  const { default: installExtension, REACT_DEVELOPER_TOOLS } = require('electron-devtools-installer');
  const forceDownload = true;
  await installExtension(REACT_DEVELOPER_TOOLS, forceDownload);
}