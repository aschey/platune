import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes, Intent, Alert } from '@blueprintjs/core';
import { FolderView } from './FolderView';
import { Dialog } from './Dialog';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);
    const [canClose, setCanClose] = useState<boolean>(true);
    const [rows, setRows] = useState<Array<string>>([]);
    const [alertOpen, setAlertOpen] = useState<boolean>(false);

    const settingsMenu = (
        <Menu>
            <MenuItem text='Configure Folders' icon='folder-open' onClick={() => setIsOpen(true)}/>
        </Menu>
    );

    const onClose = () => {
        if (canClose) { 
            setIsOpen(false);
        }
        else {
            setAlertOpen(true);
        }
    }

    const onAlertConfirm = () => {
        setAlertOpen(false);
        setCanClose(true);
        setIsOpen(false);
    }

    const onAlertCancel = () => {
        setAlertOpen(false);
        setCanClose(false);
        setIsOpen(true);
    }

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
                onClose={onClose}
                autoFocus={true}
                enforceFocus={true}
                usePortal={true}>
                <FolderView width={950} height={height} rows={rows} setRows={setRows} setCanClose={setCanClose}/>
            </Dialog>
            <Alert intent={Intent.DANGER} isOpen={alertOpen} className={`bp3-dark`} onConfirm={onAlertConfirm} confirmButtonText='Discard' cancelButtonText='Cancel' onCancel={onAlertCancel}>
            You have unsaved changes
        </Alert>
        </>
    )
}