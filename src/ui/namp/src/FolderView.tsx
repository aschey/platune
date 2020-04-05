import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button } from '@blueprintjs/core';

export const FolderView: React.FC<{}> = () => {
    const [rows, setRows] = useState<Array<string>>([]);
    
    useEffect(() => {
        const options = {
            method: 'GET',
            headers: {
              'Accept': 'application/json',
              'Content-Type': 'application/json;charset=UTF-8'
            }
          };
        fetch("http://localhost:5000/configuredFolders", options)
          .then(async response => {
             const data = await response.json();
             setRows(data);
            });
    }, []);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };
    return (
        <>
        <Button text="Select"/>
        <Table numRows={rows.length}>
            <Column name="Path" cellRenderer={cellRenderer}/>
        </Table>
        </>
    )
}