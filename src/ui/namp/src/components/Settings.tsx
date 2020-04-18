import React, { useState, useEffect } from 'react';
import { Menu, MenuItem, Popover, Button, Classes, Intent, Alert, ButtonGroup, Tabs, Tab, TabId, Text, Tooltip, Icon, Divider } from '@blueprintjs/core';
import { FolderView } from './FolderView';
import { Dialog } from './Dialog';
import { DirtyCheck } from './DirtyCheck';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { PathPicker } from './PathPicker';
import { MultilineText } from './MultilineText';

export const Settings: React.FC<{}> = () => {
    const [isOpen, setIsOpen] = useState<boolean>(false);
    const [rows, setRows] = useState<Array<string>>([]);
    const [originalRows, setOriginalRows] = useState<Array<string>>([]);
    const [selectedTab, setSelectedTab] = useState<TabId>('f');
    const [chosenTab, setChosenTab] = useState<TabId>('f');
    const [canCloseFolders, setCanCloseFolders] = useState<boolean>(true);
    const [canCloseDbPath, setCanCloseDbPath] = useState<boolean>(true);
    const [alertOpen, setAlertOpen] = useState<boolean>(false);

    const [originalPath, setOriginalPath] = useState<string>('');
    const [path, setPath] = useState<string>('');

    const mapping: Record<TabId, boolean> = {
        'f': canCloseFolders,
        't': canCloseDbPath
    }

    const onTabChange = (newTab: TabId) => {
        setChosenTab(newTab);
        if (mapping[selectedTab]) { 
            setSelectedTab(newTab) 
        } 
        else { 
            setAlertOpen(true);
        }
    }

    const height = Math.round(window.innerHeight * .66);
    const width = Math.round(window.innerWidth * .8);
    const innerWidth = Math.round((width - 260) * .9);
    const headerAndMargin = 60;
    const tabHeight = height-headerAndMargin;
    const buttonHeight = 30;
    const buttonPanelHeight = 50;

    const configureFolders = 
        <DirtyCheck 
            originalVal={originalRows} 
            newVal={rows} 
            alertOpen={alertOpen} 
            setAlertOpen={setAlertOpen} 
            canClose={canCloseFolders} 
            setCanClose={setCanCloseFolders} 
            onAlertConfirm={() => setSelectedTab(chosenTab)}>
            <FolderView 
                width={innerWidth} 
                height={tabHeight} 
                buttonHeight={buttonHeight} 
                buttonPanelHeight={buttonPanelHeight}
                rows={rows} 
                setRows={setRows} 
                setOriginalRows={setOriginalRows}/>
        </DirtyCheck>

    const chooseDatabase = 
        <DirtyCheck
            originalVal={originalPath}
            newVal={path}
            alertOpen={alertOpen}
            setAlertOpen={setAlertOpen}
            canClose={canCloseDbPath}
            setCanClose={setCanCloseDbPath}
            onAlertConfirm={() => setSelectedTab(chosenTab)}>
            <PathPicker 
                width={innerWidth} 
                buttonHeight={buttonHeight} 
                marginBottom={buttonPanelHeight}
                height={tabHeight}
                setOriginalPath={setOriginalPath}
                path={path}
                setPath={setPath}/>
        </DirtyCheck>
    return (
        <>
            <Tooltip content='Settings' hoverOpenDelay={500}>
                <Button minimal icon='cog' onClick={() => setIsOpen(true)}/>
            </Tooltip>
            
            <Dialog
                style={{width: width, height: height}}
                icon='cog' 
                title='Settings' 
                isOpen={isOpen} 
                onClose={() => setIsOpen(false)}
                autoFocus={true}
                enforceFocus={true}
                usePortal={true}>
                <div style={{paddingLeft: 10, height: height}}>
                    <Tabs vertical selectedTabId={selectedTab} onChange={onTabChange} renderActiveTabPanelOnly>
                        <Tab id='f' title={<MultilineText maxWidth={200} icon='folder-open' text='Configure Folders'/> } panel={configureFolders}/>
                        <Tab id='t' title={<MultilineText maxWidth={200} icon='database' text='Choose Database Path'/>} panel={chooseDatabase}/>
                        <Tab id='m' title={<MultilineText maxWidth={200} icon='arrows-horizontal' text='Configure Path Mappings'/>}/>
                    </Tabs>
                </div>
            </Dialog>
        </>
    )
}