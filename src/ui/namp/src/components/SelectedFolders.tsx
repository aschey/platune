import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, AnchorButton, Intent } from '@blueprintjs/core';

interface SelectedFoldersProps {
    rows: string[];
    setRows: (rows: string[]) => void;
    width: string;
    height: string;
}

export const SelectedFolders: React.FC<SelectedFoldersProps> = ({ rows, setRows, width, height }: SelectedFoldersProps) => {
    const deleteClick = (rowIndex: number) => {
        const newRows = rows.filter((r, i) => i !== rowIndex);
        setRows([...newRows]);
    }

    const pathRenderer = (rowIndex: number) => <Cell style={{fontSize: 14}}>{rows[rowIndex]}</Cell>;

    const deleteRenderer = (rowIndex: number) => 
        <Cell style={{padding: '0 2px'}}><Button intent={Intent.DANGER} icon='delete' onClick={() => deleteClick(rowIndex)} minimal small/></Cell>;
    
    return (
        <div style={{width, height}}>
        <Table numRows={rows.length} columnWidths={[300, 30]} rowHeights={rows.map(() => 25)}>
            <Column name='Path' cellRenderer={pathRenderer}/>
            <Column name='' cellRenderer={deleteRenderer}/>
        </Table>
        </div>
    )
}