import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes } from '@blueprintjs/core';
import { FolderView } from './FolderView';
import { Dialog } from './Dialog';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);

    const settingsMenu = (
        <Menu>
            <MenuItem text='Configure Folders' icon='folder-open' onClick={() => setIsOpen(true)}/>
        </Menu>
    );
    const height = Math.round(window.innerHeight * .66);
    return (
        <>
            <Popover content={settingsMenu}>
                <Button className={Classes.MINIMAL} icon='cog' rightIcon='caret-down' />
            </Popover>
            <Dialog 
                style={{width: 1000, height }}
                icon='folder-open' 
                title='Configure Folders' 
                isOpen={isOpen} 
                onClose={() => setIsOpen(false)}
                autoFocus={true}
                enforceFocus={true}
                usePortal={true}>
                <FolderView width={950} height={height}/>
            </Dialog>
        </>
    )
}