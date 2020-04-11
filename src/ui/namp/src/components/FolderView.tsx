import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent, Toaster, Toast, ButtonGroup, Divider } from '@blueprintjs/core';
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
        AppToaster.show({message: 'Success', intent: Intent.SUCCESS, icon: 'tick-circle', timeout: 1000});
    }

    const revertClick = () => {
        refreshFolders();
    }

    return (
        <div style={{display: 'flex', alignItems: 'top', alignSelf: 'center', width, height, marginTop: 20}}>
            <div style={{display: 'flex', flexDirection: 'column', width: width * .5}}>
                <SelectedFolders rows={rows} setRows={setRows} width={width * .5} height={height-50}/>
                <div style={{display: 'flex', flexDirection: 'column'}}>
                <div style={{margin: '5px'}}/>
                <div style={{display: 'flex', flexDirection: 'row'}}>
                <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' 
                            onClick={saveFoldersClick}/>
                <div style={{margin: '5px'}}/>
                <Button intent={Intent.WARNING} icon='undo' text='Revert' onClick={revertClick}/>
                </div>
            </div>
            </div>
            <div style={{margin: '5px'}}/>
            <div style={{display: 'flex', flexDirection: 'column', width: width * .5}}>
                <FolderPicker setSelected={setSelected} height={height-50}/>
                <div style={{margin: '5px'}}/>
                <div style={{display: 'flex', flexDirection: 'row'}}>
                    <Button intent={Intent.PRIMARY} onClick={addFolderClick} icon='add' text='Add'/>
                </div>
            </div>
        </div>
    )
}
