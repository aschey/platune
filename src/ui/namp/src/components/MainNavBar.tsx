import React, { useState, useEffect } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position, Icon, ButtonGroup, Intent } from '@blueprintjs/core';
import { Settings } from './Settings';
import luneDark from '../res/lune-text-dark.png';
import lune from '../res/lune-text.png';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';

interface MainNavBarProps {
    setSelectedGrid: (grid: string) => void;
    selectedGrid: string;
    theme: string;
}
export const MainNavBar: React.FC<MainNavBarProps> = ({selectedGrid, setSelectedGrid, theme}) => {
    return (
        <Navbar fixedToTop style={{height: '40px'}}>
            <NavbarGroup align={Alignment.LEFT} style={{height: 40, paddingTop: 1}}>
                <NavbarHeading style={{marginRight: 0, marginTop: 4, paddingRight: 7}}><img src={theme === 'dark' ? lune : luneDark} width={92} height={28}/></NavbarHeading>
                <NavbarDivider />
                <Settings/>
            </NavbarGroup>
            <NavbarGroup align={Alignment.RIGHT} style={{height: 40, paddingTop: 1}}>
                <ButtonGroup minimal>
                    <Button outlined intent={selectedGrid === 'song' ? Intent.PRIMARY : Intent.NONE} icon='list' onClick={() => setSelectedGrid('song')}/>
                    <Button outlined intent={selectedGrid === 'album' ? Intent.PRIMARY : Intent.NONE} icon='list-detail-view' onClick={() => setSelectedGrid('album')}/>
                </ButtonGroup>
            </NavbarGroup>
        </Navbar>
    );
}