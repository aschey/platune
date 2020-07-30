import React, { useState, useEffect } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position, Icon, ButtonGroup, Intent } from '@blueprintjs/core';
import { Settings } from './Settings';
import luneDark from '../res/lune-text-dark.png';
import lune from '../res/lune-text.png';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faThList, faTimes } from '@fortawesome/free-solid-svg-icons'
import { faSquare, faWindowMinimize, faWindowClose } from '@fortawesome/free-regular-svg-icons'
import { BrowserWindow, remote } from 'electron';

interface MainNavBarProps {
    setSelectedGrid: (grid: string) => void;
    selectedGrid: string;
    isLightTheme: boolean;
    updateTheme: (newThemeName: string) => void
}
export const MainNavBar: React.FC<MainNavBarProps> = ({ selectedGrid, setSelectedGrid, isLightTheme, updateTheme }) => {
    const getWindow = () => remote.BrowserWindow.getFocusedWindow();
    return (
        <Navbar fixedToTop style={{ height: '40px', paddingRight: 5 }}>
            <NavbarGroup align={Alignment.LEFT} style={{ height: 40, paddingTop: 1 }}>
                <NavbarHeading style={{ marginRight: 0, marginTop: 4, paddingRight: 7 }}><img src={isLightTheme ? luneDark : lune} width={92} height={28} /></NavbarHeading>
                <NavbarDivider />
                <Button minimal icon='menu' />
                <div style={{ width: 5 }} />
                <Settings updateTheme={updateTheme} />
            </NavbarGroup>
            <NavbarGroup align={Alignment.RIGHT} style={{ height: 40, paddingTop: 1 }}>
                <ButtonGroup minimal>
                    <Button
                        outlined
                        style={{ width: 30, height: 30 }}
                        intent={selectedGrid === 'song' ? Intent.PRIMARY : Intent.NONE}
                        onClick={() => setSelectedGrid('song')}>
                        <FontAwesomeIcon icon={faThList} style={{ marginTop: 1.5 }} />
                    </Button>
                    <div style={{ width: 3 }}></div>
                    <Button
                        outlined
                        intent={selectedGrid === 'album' ? Intent.PRIMARY : Intent.NONE}
                        icon='list-detail-view'
                        onClick={() => setSelectedGrid('album')}
                    />
                </ButtonGroup>
                <div style={{ width: 20 }} />
                <ButtonGroup minimal>
                    <Button intent={Intent.WARNING} className='hover-intent' onClick={() => getWindow()?.minimize()}>
                        <FontAwesomeIcon icon={faWindowMinimize} />
                    </Button>
                    <Button intent={Intent.SUCCESS} className='hover-intent' style={{transform: 'translate(0, 1px)'}} onClick={() => {
                        const window = getWindow();
                        if (window?.isMaximized()) {
                            window?.restore();
                        }
                        else {
                            window?.maximize();
                        }
                        window?.reload();
                    }}>
                        <FontAwesomeIcon icon={faSquare} />
                    </Button>
                    <Button intent={Intent.DANGER} className='hover-intent' onClick={() => getWindow()?.close()}>
                        <FontAwesomeIcon icon={faTimes} style={{transform: 'translate(0, 1px)'}} />
                    </Button>
                </ButtonGroup>
            </NavbarGroup>
        </Navbar>
    );
}