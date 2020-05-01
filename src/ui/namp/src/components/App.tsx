import React from 'react';
import logo from './logo.svg';
import '../css/App.css';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';
import { SongGrid } from './SongGrid';
import { ipcRenderer } from 'electron';

export interface ITreeState {
  nodes: ITreeNode[];
  homeDir: string;
}


const App: React.FC<{}> = () => {

    //console.log(ipcRenderer.sendSync('synchronous-message', 'ping')) // prints "pong"
    
    ipcRenderer.on('test', (event: any, arg: any) => {
      console.log(arg) // prints "pong"
    })
    ipcRenderer.send('asynchronous-message', 'ping')
    return (
        <div className="bp3-dark">
            <MainNavBar/>
        <div style={{paddingTop: 50}}>
        <SongGrid/>
        </div>
        
    </div>
    );
}

export default App;

