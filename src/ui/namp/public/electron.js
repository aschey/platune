const electron = require('electron');
const app = electron.app;
const BrowserWindow = electron.BrowserWindow;

const path = require('path');
const isDev = require('electron-is-dev');
const { spawn } = require('child_process');

let mainWindow;

function createWindow() {
    mainWindow = new BrowserWindow({width: 900, height: 680, icon: path.join(__dirname, '../src/res/logo.png'), webPreferences: { 
        webSecurity: !isDev, 
        nodeIntegration: true, 
        nodeIntegrationInWorker: true 
    }});
    mainWindow.loadURL(isDev ? 'http://localhost:3000' : `file://${path.join(__dirname, '../build/index.html')}`);
    
    mainWindow.on('closed', () => mainWindow = null);
}

app.on('ready', () => {
    var zmq = require('zeromq')
    , sock = zmq.socket('push');

    sock.bindSync('tcp://127.0.0.1:8001');
    console.log('Producer bound to port 3000');
    let proc = spawn('.\\target\\debug\\namp.exe', {cwd: '../../..', detached: true, windowsHide: true, shell: isDev, stdio: 'ignore'});
    proc.unref();
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