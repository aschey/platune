import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion } from '@blueprintjs/table';
import { Text, Label } from '@blueprintjs/core';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { range, sleep } from '../util';
import { Audio } from './Audio';

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const playingRow = useRef<number>(-1);
    const [songQueue, setSongQueue] = useState<string[]>([]);
    const numTries = 10;

    const loadSongs = async () => {
        for (let i of range(numTries)) {
            try {
                const songs = await getJson<Song[]>('/songs');
                return songs;
            }
            catch (e) {
                if (i === numTries - 1) {
                    throw e;
                }
                await sleep(1000);
            }
        }
        return [];
    }

    useEffect(() => {
        loadSongs().then(setSongs);
    }, []);

    const onSongFinished = () => {
        setSongQueue([songs[playingRow.current + 2].path]);
        playingRow.current++;
    }

    const onSelection = (p: IRegion[]) => {
        if (p.length > 0 && p[0] !== null && p[0] !== undefined && p[0].rows !== null && p[0].rows !== undefined) {
            const songIndex = p[0].rows[0];
            playingRow.current = songIndex;
            setSongQueue([songs[songIndex].path, songs[songIndex + 1].path]);
        }
    }

    return (
        <>
            <Table 
                numRows={songs.length} 
                selectionModes={SelectionModes.ROWS_AND_CELLS} 
                forceRerenderOnSelectionChange={true} 
                selectedRegionTransform={(region, event) => ({rows: region.rows})} 
                onSelection={onSelection}>
                <Column name='title' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].name}</Text></Cell>}/>
                <Column name='album artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].albumArtist}</Text></Cell>}/>
                <Column name='artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].artist}</Text></Cell>}/>
                <Column name='album' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].album}</Text></Cell>}/>
                <Column name='path' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].path}</Text></Cell>}/>
            </Table>
            <Audio songQueue={songQueue} onFinished={onSongFinished}/>
        </>
    )
}