const path = require('path');
const worker = new Worker(path.resolve(__dirname, '../workers/tcpServer.js'));