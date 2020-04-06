import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson } from './fetchUtil';

export const FolderView: React.FC<{}> = () => {
    const [rows, setRows] = useState<Array<string>>([]);
    
    useEffect(() => {
      getJson<Array<string>>('/configuredFolders').then(setRows);
    }, []);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };
    return (
        <div style={{display: 'flex', alignItems: 'center'}}>
        <Table numRows={rows.length}>
            <Column name="Path" cellRenderer={cellRenderer}/>
        </Table>
        <FolderPicker/>
        </div>
    )
}
