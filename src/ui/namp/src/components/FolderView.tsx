import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent, Toaster, Toast, ButtonGroup, Divider, Dialog, Alert } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson, putJson } from '../fetchUtil';
import { SelectedFolders } from './SelectedFolders';

const AppToaster = Toaster.create({
    position: Position.TOP
});


export const FolderView: React.FC<{width: number, height: number}> = ({width, height}) => {
    const [rows, setRows] = useState<Array<string>>([]);
    const [selected, setSelected] = useState<string>('');
    
    useEffect(() => {
      refreshFolders();
    }, []);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };

    const refreshFolders = () => getJson<Array<string>>('/configuredFolders').then(setRows);

    const addFolderClick = () => {
        rows.push(selected);
        setRows([...rows]);
    }

    const saveFoldersClick = async () => {
        await putJson<void>('/updateFolders', {folders: rows});
        AppToaster.show({message: 'Error', intent: Intent.DANGER, icon: 'tick-circle', timeout: 0});
    }

    const revertClick = () => {
        refreshFolders();
    }
    const spacerWidth = 5;
    const panelWidth = (width - spacerWidth) / 2;
    return (
        <>
        <div style={{display: 'flex', alignItems: 'top', alignSelf: 'center', width, height, marginTop: 20}}>
            <div style={{display: 'flex', flexDirection: 'column', width: panelWidth}}>
                <SelectedFolders rows={rows} setRows={setRows} width={panelWidth} height={height-50}/>
                <div style={{display: 'flex', flexDirection: 'column'}}>
                <div style={{margin: spacerWidth}}/>
                <div style={{display: 'flex', flexDirection: 'row'}}>
                <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' 
                            onClick={saveFoldersClick}/>
                <div style={{margin: spacerWidth}}/>
                <Button intent={Intent.WARNING} icon='undo' text='Revert' onClick={revertClick}/>
                </div>
            </div>
            </div>
            <div style={{ width: spacerWidth}}/>
            <div style={{display: 'flex', flexDirection: 'column', width: panelWidth}}>
                <FolderPicker setSelected={setSelected} width={panelWidth} height={height-50}/>
                <div style={{margin: spacerWidth}}/>
                <div style={{display: 'flex', flexDirection: 'row'}}>
                    <Button intent={Intent.PRIMARY} onClick={addFolderClick} icon='add' text='Add'/>
                </div>
            </div>
        </div>
        <Alert intent={Intent.DANGER} isOpen={false} className={`bp3-dark`} >
        <p>
                        Couldn't create the file because the containing folder doesn't exist anymore. You will be
                        redirected to your user folder.
                    </p>
        </Alert>
    </>
    )
}
