const net = require('net');
console.log('here');
net.createServer().listen({
    host: 'localhost',
    port: 8001,
    exclusive: true
});