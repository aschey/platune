// define child rescript
var webpack = require('webpack');
const {appendWebpackPlugin} = require('@rescripts/utilities');
module.exports = config => {
    config.target = 'electron-renderer';
    //config = appendWebpackPlugin(new webpack.IgnorePlugin(/zeromq/), config);
    return config;
  }