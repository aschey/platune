import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson } from './fetchUtil';
import { SelectedFolders } from './SelectedFolders';

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
        <div style={{display: 'flex', alignItems: 'center'}}>
        <SelectedFolders rows={rows}/>
        <FolderPicker setSelected={setSelected}/>
        <Button intent={Intent.PRIMARY} onClick={addFolderClick} text="Add"/>
        </div>
    )
}
