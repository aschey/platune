import React, { useState, useEffect } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion } from '@blueprintjs/table';
import { Text } from '@blueprintjs/core';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
var Sound = require('react-sound').default;

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [selectedRow, setSelectedRow] = useState<number>(-1);

    useEffect(() => {
        getJson<Song[]>('/songs').then(setSongs);
    }, []);
    const titleRenderer = (rowIndex: number) => {
        return (
            <Cell>
                <Text>
                    {songs[rowIndex].name}
                </Text>
                <Sound
                    volume={30}
                    url={songs[rowIndex].path}
                    playStatus={rowIndex === selectedRow ? Sound.status.PLAYING : Sound.status.STOPPED}
                />
            </Cell>
        )
    }
    return (
        <Table numRows={songs.length} selectionModes={SelectionModes.ROWS_ONLY} onSelection={(p: IRegion[]) => {
                if (p.length > 0 && p[0] !== null && p[0] !== undefined && p[0].rows !== null && p[0].rows !== undefined) {
                    setSelectedRow(p[0].rows[0]);
                }
            }
            }>
            <Column name='title' cellRenderer={titleRenderer}/>
            <Column name='artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].artist}</Text></Cell>}/>
        </Table>
    )
}