import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes, Intent, Alert } from '@blueprintjs/core';
import { FolderView } from './FolderView';
import { Dialog } from './Dialog';
import { DirtyCheckDialog } from './DirtyCheckDialog';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);
    const [rows, setRows] = useState<Array<string>>([]);
    const [originalRows, setOriginalRows] = useState<Array<string>>([]);

    const settingsMenu = (
        <Menu>
            <MenuItem text='Configure Folders' icon='folder-open' onClick={() => setIsOpen(true)}/>
        </Menu>
    );

    const arraysEqual = (a: string[], b: string[]): boolean => {
        if (a === b) return true;
        if (a == null || b == null) return false;
        if (a.length !== b.length) return false;
    
        const sortedA = a.concat().sort();
        const sortedB = b.concat().sort();
        for (var i = 0; i < sortedA.length; ++i) {
            if (sortedA[i] !== sortedB[i]) return false;
        }
        return true;
    }

    const height = Math.round(window.innerHeight * .66);
    return (
        <>
            <Popover content={settingsMenu}>
                <Button className={Classes.MINIMAL} icon='cog' rightIcon='caret-down' />
            </Popover>
            <DirtyCheckDialog<string[]>
                style={{width: 1000, height }}
                originalVal={originalRows}
                newVal={rows}
                isOpen={isOpen}
                setIsOpen={setIsOpen}
                checkEqual={arraysEqual}>
                <FolderView width={950} height={height} rows={rows} setRows={setRows} setOriginalRows={setOriginalRows}/>
            </DirtyCheckDialog>
        </>
    )
}