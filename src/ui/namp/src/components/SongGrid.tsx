import React, { useState, useEffect } from 'react';
import { Table, Cell, Column } from '@blueprintjs/table';
var Sound = require('react-sound').default;

export const SongGrid: React.FC<{}> = () => {
    const cellRenderer = (rowIndex: number) => {
        return (
            <Cell>
                <Sound
                    volume={30}
                    url="http://localhost:5000/home/aschey/windows/shared_files/Music/A Lot Like Birds/No Place/A Lot Like Birds - No Place - 01 In Trances.mp3"
                    playStatus={Sound.status.PLAYING}
                />
            </Cell>
        )
    }
    return (
        <Table numRows={1}>
            <Column name="Test" cellRenderer={cellRenderer}/>
        </Table>
    )
}