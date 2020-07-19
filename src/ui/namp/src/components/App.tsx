import React, { useState } from 'react';
import logo from './logo.svg';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';
import { ipcRenderer } from 'electron';
import { SongGrid } from './SongGrid';
import { applyTheme } from '../themes/themes';

export interface ITreeState {
  nodes: ITreeNode[];
  homeDir: string;
}

const App: React.FC<{theme: string}> = ({theme}) => {
    const [selectedGrid, setSelectedGrid] = useState('song');
    return (
        <>
            <MainNavBar selectedGrid={selectedGrid} setSelectedGrid={setSelectedGrid} theme={theme}/>
            <div style={{paddingTop: 40}}>
                <SongGrid selectedGrid={selectedGrid}/>
            </div>
        </>
    );
}

export default App;

