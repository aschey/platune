import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent, Toaster, Toast, ButtonGroup, Divider } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson } from '../fetchUtil';
import { SelectedFolders } from './SelectedFolders';
import { intentClass } from '@blueprintjs/core/lib/esm/common/classes';

const AppToaster = Toaster.create({
    position: Position.TOP
});


export const FolderView: React.FC<{}> = () => {
    const [rows, setRows] = useState<Array<string>>([]);
    const [selected, setSelected] = useState<string>('');
    
    useEffect(() => {
      getJson<Array<string>>('/configuredFolders').then(setRows);
    }, []);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };

    const addFolderClick = () => {
        rows.push(selected);
        setRows([...rows]);
    }

    return (
        <div style={{display: 'flex', alignItems: 'top', height: '500px', marginTop: '20px'}}>
            <SelectedFolders rows={rows} setRows={setRows} width='400px' height='380px'/>
            <div style={{display: 'flex', flexDirection: 'column'}}>
                <FolderPicker setSelected={setSelected} width='500px' height='380px'/>
                <div style={{margin: '5px'}}/>
                <div style={{display: 'flex', flexDirection: 'row'}}>
                    <Button intent={Intent.PRIMARY} onClick={addFolderClick} icon='add' text='Add'/>
                    <div style={{margin: '5px'}}/>
                    <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' 
                        onClick={() => AppToaster.show({message: 'Success', intent: Intent.SUCCESS, icon: 'tick-circle', timeout: 1000})}/>
                </div>
            </div>
        </div>
    )
}
