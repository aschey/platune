// define child rescript
var webpack = require('webpack');
var BowerWebpackPlugin = require("bower-webpack-plugin");
const {appendWebpackPlugin} = require('@rescripts/utilities');

module.exports = config => {
    config.target = 'electron-renderer';
    return config;
  }