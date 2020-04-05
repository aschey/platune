import React, { useState, useEffect } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position, Icon } from '@blueprintjs/core';
import { Settings } from './Settings';

export const MainNavBar: React.FC<{}> = () => {

    return (
        <Navbar fixedToTop style={{height: '40px'}}>
            <NavbarGroup align={Alignment.LEFT} style={{height: '40px'}}>
                <NavbarHeading><Icon icon="music"/> NAMP</NavbarHeading>
                <NavbarDivider />
                <Settings/>
            </NavbarGroup>
        </Navbar>
    );
}