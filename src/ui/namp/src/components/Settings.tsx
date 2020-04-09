import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes, Dialog } from '@blueprintjs/core';
import { FolderView } from './FolderView';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);

    const settingsMenu = (
        <Menu>
            <MenuItem text='Configure Folders' icon='folder-open' onClick={() => setIsOpen(true)}/>
        </Menu>
    );
    
    return (
        <>
            <Popover content={settingsMenu}>
                <Button className={Classes.MINIMAL} icon='cog' rightIcon='caret-down' />
            </Popover>
            <Dialog 
                style={{width: '1000px', boxShadow: 'none'}}
                className={`${Classes.DIALOG} bp3-dark`} 
                icon='folder-open' 
                title='Configure Folders' 
                isOpen={isOpen} 
                onClose={() => setIsOpen(false)}
                autoFocus={true}
                enforceFocus={true}
                usePortal={true}>
                <FolderView/>
            </Dialog>
        </>
    )
}