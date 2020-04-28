import React from 'react';
import logo from './logo.svg';
import '../css/App.css';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';
import { SongGrid } from './SongGrid';

export interface ITreeState {
  nodes: ITreeNode[];
  homeDir: string;
}


const App: React.FC<{}> = () => {
    return (
        <>
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'"></meta>
        <div className="bp3-dark">
            <MainNavBar/>
        <div style={{paddingTop: 50}}>
        <SongGrid/>
        </div>
        
    </div>
    </>
    );
}

export default App;

