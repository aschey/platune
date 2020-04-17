import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes, Intent, Alert, ButtonGroup, Tabs, Tab, TabId, Text, Tooltip, Icon } from '@blueprintjs/core';
import { FolderView } from './FolderView';
import { Dialog } from './Dialog';
import { DirtyCheckDialog } from './DirtyCheck';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);
    const [rows, setRows] = useState<Array<string>>([]);
    const [originalRows, setOriginalRows] = useState<Array<string>>([]);
    const [selectedTab, setSelectedTab] = useState<TabId>('f');
    const [chosenTab, setChosenTab] = useState<TabId>('f');
    const [canCloseFolders, setCanCloseFolders] = useState<boolean>(true);
    const [canCloseDbPath, setCanCloseDbPath] = useState<boolean>(true);
    const [alertOpen, setAlertOpen] = useState<boolean>(false);

    const mapping: Record<TabId, boolean> = {
        'f': canCloseFolders,
        't': canCloseDbPath
    }

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

    const onTabChange = (newTab: TabId) => {
        setChosenTab(newTab);
        if (mapping[selectedTab.toString()]) { 
            setSelectedTab(newTab) 
        } 
        else { 
            setAlertOpen(true);
        }
    }

    const height = Math.round(window.innerHeight * .66);
    const headerAndMargin = 60;

    const configureFolders = 
        <DirtyCheckDialog 
            originalVal={originalRows} 
            newVal={rows} 
            checkEqual={arraysEqual} 
            alertOpen={alertOpen} 
            setAlertOpen={setAlertOpen} 
            canClose={canCloseFolders} 
            setCanClose={setCanCloseFolders} 
            onAlertConfirm={() => setSelectedTab(chosenTab)}>
            <FolderView width={950} height={height-headerAndMargin} rows={rows} setRows={setRows} setOriginalRows={setOriginalRows}/>
        </DirtyCheckDialog>

    
    return (
        <>
            <Tooltip content='Settings' hoverOpenDelay={500}>
                <Button minimal icon='cog' onClick={() => setIsOpen(true)}/>
            </Tooltip>
            
            <Dialog
                style={{width: 1200, height: height}}
                icon='cog' 
                title='Settings' 
                isOpen={isOpen} 
                onClose={() => setIsOpen(false)}
                autoFocus={true}
                enforceFocus={true}
                usePortal={true}>
                <div style={{paddingLeft: 10}}>
                    <Tabs vertical selectedTabId={selectedTab} onChange={onTabChange} renderActiveTabPanelOnly>
                        <Tab id='f' title={<><Icon icon='folder-open'/><span style={{whiteSpace: 'pre'}}>  Configure Folders</span></>} panel={configureFolders}/>
                        <Tab id='t' title={<><Icon icon='database'/><span style={{whiteSpace: 'pre'}}>  Choose Database Path</span></>} panel={<p>test</p>}/>
                    </Tabs>
                </div>
            </Dialog>
        </>
    )
}