import { ipcMain, app, BrowserWindow } from 'electron';
import path from 'path';
import isDev from 'electron-is-dev';
import { spawn } from 'child_process';
import zmq from 'zeromq';

let mainWindow: BrowserWindow | null;

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
ipcMain.on('asynchronous-message', (event, arg) => {
  console.log(arg) // prints "ping"
  event.reply('asynchronous-reply', 'pong')
})

ipcMain.on('synchronous-message', (event, arg) => {
  console.log(arg) // prints "ping"
  event.returnValue = 'pong'
})
    var reqSock = zmq.socket('req');
    var subSock = zmq.socket('sub');
    reqSock.connect('tcp://127.0.0.1:8001');
    subSock.connect('tcp://127.0.0.1:8002');
    subSock.subscribe('group1');

  console.log("sending...");
  reqSock.send(JSON.stringify({ test: 'sure' }));
  reqSock.on('error', console.log);
  reqSock.on('message', res => {
    console.log(JSON.parse(res));
    //sock.send(JSON.stringify({ test: 'sure' }));
  });

  subSock.on('message', (topic, res) => mainWindow?.webContents.send('test', JSON.parse(res)));
    
    // let command = process.platform === 'win32' ? '.\\target\\debug\\namp.exe' : './target/debug/namp';
    // let proc = spawn(command, {cwd: '../../..', detached: true, windowsHide: true, shell: isDev, stdio: 'ignore'});
    // proc.unref();
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