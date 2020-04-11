import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, AnchorButton, Intent } from '@blueprintjs/core';

interface SelectedFoldersProps {
    rows: string[];
    setRows: (rows: string[]) => void;
    width: number;
    height: number;
}

export const SelectedFolders: React.FC<SelectedFoldersProps> = ({ rows, setRows, width, height }: SelectedFoldersProps) => {
    const deleteRowWidth = 30;

    const deleteClick = (rowIndex: number) => {
        const newRows = rows.filter((r, i) => i !== rowIndex);
        setRows([...newRows]);
    }

    const pathRenderer = (rowIndex: number) => <Cell style={{fontSize: 14}}>{rows[rowIndex]}</Cell>;

    const deleteRenderer = (rowIndex: number) => 
        <Cell style={{padding: '0 3px'}}><Button intent={Intent.DANGER} icon='delete' onClick={() => deleteClick(rowIndex)} minimal small/></Cell>;
    
    return (
        <div style={{width, height}}>
        <Table numRows={rows.length} columnWidths={[width-(deleteRowWidth * 2), deleteRowWidth]} rowHeights={rows.map(() => 25)}>
            <Column name='Path' cellRenderer={pathRenderer}/>
            <Column name='' cellRenderer={deleteRenderer}/>
        </Table>
        </div>
    )
}