import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button } from '@blueprintjs/core';

export const FolderView: React.FC<{}> = () => {
    const [numRows, setNumRows] = useState<number>(0);
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
          .then(response => {
             response.json()
                .then(data => {
                    setNumRows(data.length);
                    setRows(data);
                })
          })
    })
    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };
    return (
        <>
        <Button text="Select"/>
        <Table numRows={numRows}>
            <Column name="Path" cellRenderer={cellRenderer}/>
        </Table>
        </>
    )
}