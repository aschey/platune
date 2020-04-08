import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';

interface SelectedFoldersProps {
    rows: string[];
}

export const SelectedFolders: React.FC<SelectedFoldersProps> = ({ rows }: SelectedFoldersProps) => {

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };
    
    return <Table numRows={rows.length}>
    <Column name="Path" cellRenderer={cellRenderer}/>
</Table>
}