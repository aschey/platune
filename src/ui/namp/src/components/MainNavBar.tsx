import React, { useState, useEffect } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position, Icon } from '@blueprintjs/core';
import { Settings } from './Settings';
import logo from '../res/logo.png';
import logoDark from '../res/logo-dark.png';


export const MainNavBar: React.FC<{}> = () => {
    return (
        <Navbar fixedToTop style={{height: '40px'}}>
            <NavbarGroup align={Alignment.LEFT} style={{height: '40px'}}>
                <NavbarHeading style={{marginRight: '0px', marginTop: '3px'}}><img src={logoDark}/></NavbarHeading>
                <NavbarDivider />
                <Settings/>
            </NavbarGroup>
        </Navbar>
    );
}