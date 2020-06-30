import React, { useState } from 'react';
import logo from './logo.svg';
import '../css/App.css';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';
import { ipcRenderer } from 'electron';
import { SongGrid } from './SongGrid';

export interface ITreeState {
  nodes: ITreeNode[];
  homeDir: string;
}


const App: React.FC<{}> = () => {
    const [selectedGrid, setSelectedGrid] = useState('song');
    return (
        <div className="bp3-dark">
            <MainNavBar selectedGrid={selectedGrid} setSelectedGrid={setSelectedGrid}/>
            <div style={{paddingTop: 40}}>
                <SongGrid selectedGrid={selectedGrid}/>
            </div>
        </div>
    );
}

export default App;

