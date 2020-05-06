import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion, RowLoadingOption, TableLoadingOption, RenderMode, RowHeaderCell } from '@blueprintjs/table';
import { Text, Label, ProgressBar, Intent, Icon, Button, EditableText } from '@blueprintjs/core';
import Observer from '@researchgate/react-intersection-observer';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { range, sleep } from '../util';
import { audioQueue } from '../audio';
import { Controls } from './Controls';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { toastSuccess } from '../appToaster';

const useForceUpdate = () => {
    const [value, setValue] = useState(0);
    return () => setValue(value => ++value);
}

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [height, setHeight] = useState(window.innerHeight - 39);
    const [playingRow, setPlayingRow] = useState(-1);
    const [selectedRow, setSelectedRow] = useState(-1);
    const [editingRow, setEditingRow] = useState(-1);
    const [isPlaying, setIsPlaying] = useState(false);

    const forceUpdate = useForceUpdate();

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

    useEffect(() => {
        // Batch rendering mode seems to cause React to skip re-rendering sometimes
        // Need to use this to ensure it updates
        forceUpdate();
        setIsPlaying(playingRow > -1);
    }, [playingRow])

    const onSongFinished = (playingRow: number) => {
        setPlayingRow(playingRow + 1);
        audioQueue.start([songs[playingRow + 2].path], playingRow + 1, onSongFinished);
    }

    const onDoubleClick = (songIndex: number) => {
        if (songIndex === editingRow) {
            return;
        }
        if (editingRow > -1) {
            // save
            toastSuccess();
            setEditingRow(-1);
        }
        setPlayingRow(songIndex);
        audioQueue.stop();
        startQueue(songIndex);
    }

    const startQueue = (songIndex: number) => {
        audioQueue.start([
            songs[songIndex].path,
            songs[songIndex + 1].path
        ], songIndex, onSongFinished);
    }

    const onSelection = (p: IRegion[]) => {
        if (p.length > 0 && p[0] !== null && p[0] !== undefined && p[0].rows !== null && p[0].rows !== undefined) {
            const songIndex = p[0].rows[0];
            setSelectedRow(songIndex);
            if (songIndex === editingRow) {
                return;
            }
            if (editingRow > -1) {
                // save
                toastSuccess();
                setEditingRow(-1);
            }
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

    const trackRenderer = (rowIndex: number) => {
        let value = songs[rowIndex].track.toString();
        if (value === '0') {
            value = '';
        }
        return cellRenderer(rowIndex, value);
    }

    const pathRenderer = (rowIndex: number) => {
        let value = songs[rowIndex].path;
        return cellRenderer(rowIndex, value, false);
    }

    const cellRenderer = (rowIndex: number, value: string, canEdit: boolean = true) => {
        if (rowIndex === editingRow && canEdit) {
            return (
                <Cell intent={Intent.PRIMARY}>
                    <EditableText value={value}/>
                </Cell>);
        }

        if (rowIndex === playingRow) {
            return (
                <Cell intent={Intent.SUCCESS}>
                    <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                        <Text>{value}</Text>
                    </div>
                </Cell>);
        }
        return (
            <Cell style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a'}}>
                <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                    <Text>{value}</Text>
                </div>
            </Cell>
        );
    }

    const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album') => {
        let value = songs[rowIndex][field].toString();
        return cellRenderer(rowIndex, value);
    }

    const rowHeaderNameRenderer = (name: string, rowIndex: number | undefined) => {
        let chosenIndex = rowIndex ?? 0;
        if (chosenIndex === playingRow && isPlaying) {
            return (
                <div style={{lineHeight: 2}} onDoubleClick={() => onDoubleClick(chosenIndex)}>
                    <Icon intent={Intent.SUCCESS} icon="volume-up"/>
                </div>
            );
        }
        return (
            <div onDoubleClick={() => onDoubleClick(chosenIndex)}>
                <Text>{rowIndex}</Text> 
            </div>
            
        );
    }

    const rowHeaderRenderer = (rowIndex: number) => {
        if (rowIndex === playingRow) {
            return <RowHeaderCell index={rowIndex} nameRenderer={rowHeaderNameRenderer}/>
        }
        return <RowHeaderCell style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a'}} index={rowIndex} nameRenderer={rowHeaderNameRenderer}/>
    }

    const onPause = () => {
        audioQueue.pause();
        setIsPlaying(false);
    }

    const onPlay = () => {
        const rowToPlay = playingRow > -1 ? playingRow : selectedRow;
        setPlayingRow(rowToPlay);
        startQueue(rowToPlay);
    }

    const onStop = () => {
        audioQueue.stop();
        setPlayingRow(-1);
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
                forceRerenderOnSelectionChange={true} 
                renderMode={RenderMode.BATCH_ON_UPDATE}
                selectedRegionTransform={(region, event) => ({rows: region.rows})} 
                enableRowResizing={false}
                onColumnWidthChanged={() => {setHeight(window.innerHeight); setTimeout(() => setHeight(window.innerHeight - 90), 1);}}
                onSelection={onSelection}
                rowHeaderCellRenderer={rowHeaderRenderer}
                >
                <Column name = '' cellRenderer={editCellRenderer}/>
                <Column name='Title' cellRenderer={(rowIndex) => genericCellRenderer(rowIndex, 'name') }/>
                <Column name='Album Artist' cellRenderer={(rowIndex) => genericCellRenderer(rowIndex, 'albumArtist')}/>
                <Column name='Artist' cellRenderer={(rowIndex) => genericCellRenderer(rowIndex, 'artist')}/>
                <Column name='Album' cellRenderer={(rowIndex) => genericCellRenderer(rowIndex, 'album')}/>
                <Column name='Track' cellRenderer={(rowIndex) => trackRenderer(rowIndex)}/>
                <Column name='Path' cellRenderer={(rowIndex) => pathRenderer(rowIndex)}/>
            </Table>
        </div>
        <Controls isPlaying={isPlaying} setIsPlaying={setIsPlaying} onPause={onPause} onPlay={onPlay} onStop={onStop}/>
        </>
    )
}