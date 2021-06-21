// define child rescript
var webpack = require('webpack');

module.exports = config => {
    config.target = 'electron-renderer';
    return config;
  }