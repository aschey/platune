import React, { useState, useEffect } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position } from '@blueprintjs/core';
import { Settings } from './Settings';

interface MainNavBarProps {

}

export const MainNavBar: React.FC<MainNavBarProps> = (props: MainNavBarProps) => {
    const settingsClick = () => {

    }

    const settingsMenu = (
        <Menu>
            <MenuItem text="Configure Folders"/>
        </Menu>
    )

    return (

        <Navbar fixedToTop>
            <NavbarGroup align={Alignment.LEFT}>
                <NavbarHeading>NAMP</NavbarHeading>
                <NavbarDivider />
                <Settings/>
            </NavbarGroup>
        </Navbar>
    );
}