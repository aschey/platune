import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion, RowLoadingOption, TableLoadingOption, RenderMode } from '@blueprintjs/table';
import { Text, Label, ProgressBar, Intent } from '@blueprintjs/core';
import Observer from '@researchgate/react-intersection-observer';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { range, sleep } from '../util';
import { Audio } from './Audio';
import { Controls } from './Controls';

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [height, setHeight] = useState<number>(window.innerHeight - 39);
    const playingRow = useRef<number>(-1);
    const [songQueue, setSongQueue] = useState<string[]>([]);
    const [, setState] = useState({});
    const numTries = 10;

    const loadSongs = async () => {
        for (let i of range(numTries)) {
            try {
                const songs = await getJson<Song[]>('/songs?offset=0&limit=15000');
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

    useEffect(() => {
        // Hack to get around a weird rendering issue
        // For some reason, it seems like the height needs to cause an overflow while the songs are loaded in order for it to render correctly
        // Once the songs load, we can set the correct height
        if (songs.length) {
            setHeight(window.innerHeight - 40 - 80);
        }
        
    }, [songs]);

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
    const width = window.innerWidth;
    const remainingWidth = width - 75 - 40;
    return (
        <>
        <div style={{height: height}}>
            <Table 
                numRows={songs.length} 
                columnWidths={[remainingWidth * .2, remainingWidth * .15, remainingWidth * .15, remainingWidth * .2, 75, remainingWidth * .3]}
                selectionModes={SelectionModes.ROWS_AND_CELLS}                 
                forceRerenderOnSelectionChange={false} 
                selectedRegionTransform={(region, event) => ({rows: region.rows})} 
                enableRowResizing={false}
                onColumnWidthChanged={() => {setHeight(window.innerHeight); setTimeout(() => setHeight(window.innerHeight - 90), 1);}}
                onSelection={onSelection}>
                <Column name='Title' cellRenderer={(rowIndex) => <Cell><Observer onChange={(a, b) => console.log(rowIndex)}><Text>{songs[rowIndex].name}</Text></Observer></Cell> }/>
                <Column name='Album Artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].albumArtist}</Text></Cell>}/>
                <Column name='Artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].artist}</Text></Cell>}/>
                <Column name='Album' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].album}</Text></Cell>}/>
                <Column name='Track' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].track > 0 ? songs[rowIndex].track : ''}</Text></Cell>}/>
                <Column name='Path' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].path}</Text></Cell>}/>
            </Table>
            <Audio songQueue={songQueue} onFinished={onSongFinished}/>
        </div>
        <Controls/>
        </>
    )
}