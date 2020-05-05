import { ipcMain, app, BrowserWindow } from 'electron';
import path from 'path';
import isDev from 'electron-is-dev';
import { spawn } from 'child_process';
import zmq from 'zeromq';
import net from 'net';

app.commandLine.appendSwitch('autoplay-policy', 'no-user-gesture-required');
let mainWindow: BrowserWindow | null;
let server: net.Server | null;

function createWindow() {
    mainWindow = new BrowserWindow({width: 900, height: 680, backgroundColor: '#000', icon: path.join(__dirname, '../src/res/logo.png'), webPreferences: { 
        webSecurity: !isDev, 
        nodeIntegration: true, 
        nodeIntegrationInWorker: false,
        backgroundThrottling: false,
        autoplayPolicy: 'no-user-gesture-required'
    }});
    mainWindow.loadURL(isDev ? 'http://localhost:3000' : `file://${path.join(__dirname, '../build/index.html')}`);
    
    mainWindow.on('closed', () => {
      mainWindow = null;
      server?.close();
      server = null;
    });
}

app.on('ready', () => {
  
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