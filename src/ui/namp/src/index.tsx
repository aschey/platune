import React from 'react';
import ReactDOM from 'react-dom';
import './css/index.scss';
import App from './components/App';
import * as serviceWorker from './serviceWorker';
import { applyTheme } from './themes/themes';
const theme = 'dark';
applyTheme(theme);
ReactDOM.render(
  <React.StrictMode>
    <App theme={theme}/>
  </React.StrictMode>,
  document.getElementById('root')
);

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
