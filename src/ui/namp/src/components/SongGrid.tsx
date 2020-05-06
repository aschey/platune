import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion, RowLoadingOption, TableLoadingOption, RenderMode } from '@blueprintjs/table';
import { Text, Label, ProgressBar, Intent, Icon, Button, EditableText } from '@blueprintjs/core';
import Observer from '@researchgate/react-intersection-observer';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { range, sleep } from '../util';
import { audioQueue } from './Audio';
import { Controls } from './Controls';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { toastSuccess } from '../appToaster';

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [height, setHeight] = useState(window.innerHeight - 39);
    const [playingRow, setPlayingRow] = useState(-1);
    const [editingRow, setEditingRow] = useState(-1);

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

    

    const onSongFinished = (playingRow: number) => {
        console.log(playingRow + 1);
        setPlayingRow(playingRow + 1);
        audioQueue.scheduleAll([songs[playingRow + 2].path], playingRow + 1, onSongFinished);
    }

    const onSelection = (p: IRegion[]) => {
        if (p.length > 0 && p[0] !== null && p[0] !== undefined && p[0].rows !== null && p[0].rows !== undefined) {
            
            const songIndex = p[0].rows[0];
            if (songIndex === editingRow) {
                return;
            }
            if (editingRow > -1) {
                // save
                toastSuccess();
                setEditingRow(-1);
            }
            setPlayingRow(songIndex);
 
            audioQueue.scheduleAll([
                songs[songIndex].path,
                songs[songIndex + 1].path
            ], songIndex, onSongFinished);
        }
    }

    const editCellRenderer = (rowIndex: number) => {
        const isEditingRow = editingRow === rowIndex;
        return (
        <Cell style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a', padding: 0, margin: 0}}>
            <FlexCol>
                <Button small minimal intent={isEditingRow || rowIndex === playingRow ? Intent.SUCCESS : Intent.NONE} icon={isEditingRow ? 'saved' : 'edit'} onClick={() => {
                    if (isEditingRow) {
                        // save
                        toastSuccess();
                        setEditingRow(-1);
                    }
                    else {
                        setEditingRow(rowIndex);
                    }
                }}/>
            </FlexCol>
        </Cell>);
    }

    const cellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album' | 'track' | 'path') => {
        let value = songs[rowIndex][field].toString();
        if (field === 'track' && value === '0') {
            value = '';
        }
        if (rowIndex === editingRow && field !== 'path') {
            return (
                <Cell intent={Intent.PRIMARY}>
                    <EditableText value={value}/>
                </Cell>);
        }
        if (rowIndex === playingRow) {
            return (
                <Cell intent={Intent.SUCCESS}>
                    <Text>{value}</Text>
                </Cell>);
        }
        return (
        <Cell style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a'}}>
            <Text>{value}</Text>
        </Cell>);
    }

    const width = window.innerWidth;
    const remainingWidth = width - 30 - 55 - 40;
    return (
        <>
        <div style={{height: height, overflowX: 'scroll'}}>
            <Table 
                numRows={songs.length} 
                rowHeights={songs.map(() => 25)}
                columnWidths={[30, remainingWidth * .2, remainingWidth * .15, remainingWidth * .15, remainingWidth * .2, 55, remainingWidth * .3]}
                selectionModes={SelectionModes.ROWS_AND_CELLS}                 
                forceRerenderOnSelectionChange={false} 
                selectedRegionTransform={(region, event) => ({rows: region.rows})} 
                enableRowResizing={false}
                onColumnWidthChanged={() => {setHeight(window.innerHeight); setTimeout(() => setHeight(window.innerHeight - 90), 1);}}
                onSelection={onSelection}>
                <Column name = ''  cellRenderer={editCellRenderer}/>
                <Column name='Title' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'name') }/>
                <Column name='Album Artist' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'albumArtist')}/>
                <Column name='Artist' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'artist')}/>
                <Column name='Album' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'album')}/>
                <Column name='Track' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'track')}/>
                <Column name='Path' cellRenderer={(rowIndex) => cellRenderer(rowIndex, 'path')}/>
            </Table>
        </div>
        <Controls/>
        </>
    )
}