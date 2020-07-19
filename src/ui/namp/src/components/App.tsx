import React, { useState } from 'react';
import logo from './logo.svg';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';
import { ipcRenderer } from 'electron';
import { SongGrid } from './SongGrid';
import { applyTheme } from '../themes/themes';
import { lightTheme } from '../themes/light';
import { darkTheme } from '../themes/dark';
import { isLight } from '../themes/colorMixer';

const themeName = 'light';
const theme = lightTheme;
applyTheme(themeName);

const App: React.FC<{}> = () => {
    const [selectedGrid, setSelectedGrid] = useState('song');
    const [themeDetails, setThemeDetails] = useState(isLight(theme.backgroundMain));
    const updateTheme = (newThemeName: string) => {
        applyTheme(newThemeName);
        const newTheme = newThemeName === 'light' ? lightTheme : darkTheme;
        setThemeDetails(isLight(newTheme.backgroundMain));
    }
    return (
        <>
            <MainNavBar selectedGrid={selectedGrid} setSelectedGrid={setSelectedGrid} isLightTheme={themeDetails} updateTheme={updateTheme}/>
            <div style={{paddingTop: 40}}>
                <SongGrid selectedGrid={selectedGrid} isLightTheme={themeDetails}/>
            </div>
        </>
    );
}

export default App;

