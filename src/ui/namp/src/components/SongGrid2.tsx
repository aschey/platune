import React, { useState, useEffect } from "react";
import { Column, Table, TableHeaderRenderer, TableHeaderProps, defaultTableRowRenderer, TableRowProps, RowMouseEventHandlerParams } from "react-virtualized";
import Draggable from "react-draggable";
import { Song } from "../models/song";
import { range, sleep } from "../util";
import { getJson } from "../fetchUtil";
import _ from "lodash";
import { Cell } from "@blueprintjs/table";
import { Intent, EditableText, Text, Button } from "@blueprintjs/core";
import { toastSuccess } from "../appToaster";
import { audioQueue } from "../audio";
import { Controls } from "./Controls";
import { FlexCol } from "./FlexCol";

export const Demo: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [height, setHeight] = useState(window.innerHeight - 39);
    const [playingRow, setPlayingRow] = useState(-1);
    const [selectedRow, setSelectedRow] = useState(-1);
    const [editingRow, setEditingRow] = useState(-1);
    const [isPlaying, setIsPlaying] = useState(false);
    const [widths, setWidths] = useState({
        edit: 30,
        name: 300,
        albumArtist: 250,
        artist: 250,
        album: 250,
        track: 60,
        path: 400
    });

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
        setIsPlaying(playingRow > -1);
    }, [playingRow])

    const headerRenderer = (props: TableHeaderProps) => {
        return (
            <>
                <div className="ReactVirtualized__Table__headerTruncatedText">
                    {props.label}
                </div>
                <Draggable
                    axis='none'
                    defaultClassName="DragHandle"
                    defaultClassNameDragging="DragHandleActive"
                    onDrag={(event, { deltaX }) => {
                        resizeRow({dataKey: props.dataKey, deltaX});
                    }}
                >
                    <span className="DragHandleIcon" style={{transform: 'translate(5px, 0) !important'}}>⋮</span>
                </Draggable>
            </>
        );
    };


    const editCellRenderer = (rowIndex: number) => {
        const isEditingRow = editingRow === rowIndex;
        return (
        <div className='bp3-table-cell gridCell striped' style={{padding: 0}}>
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
        </div>);
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

    const onSongFinished = (playingRow: number) => {
        setPlayingRow(playingRow + 1);
        audioQueue.start([songs[playingRow + 2].path], playingRow + 2, onSongFinished);
    }

    const cellRenderer = (rowIndex: number, value: string, canEdit: boolean = true) => {
        if (rowIndex === editingRow) {
            return (
                <div className='bp3-table-cell bp3-intent-primary gridCell' onDoubleClick={() => onDoubleClick(rowIndex)}>
                    { canEdit ? <EditableText defaultValue={value}/> : value }
                </div>);
        }

        if (rowIndex === playingRow) {
            return (
                <div className='bp3-table-cell bp3-intent-success gridCell'onDoubleClick={() => onDoubleClick(rowIndex)}>
                    {value}
                </div>);
        }
        if (rowIndex === selectedRow) {
            return (
                <div className='bp3-table-cell gridCell' onDoubleClick={() => onDoubleClick(rowIndex)}>
                    {value}
                </div>
            );
        }
        return (
            <div className='bp3-table-cell striped gridCell' onDoubleClick={() => onDoubleClick(rowIndex)}>
                {value}
            </div>
        );
    }

    const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album') => {
        let value = songs[rowIndex][field].toString();
        return cellRenderer(rowIndex, value);
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

    const resizeRow = (props: {dataKey: string, deltaX: number}) => {
        const newWidths: any =_.cloneDeep(widths);
        
        newWidths[props.dataKey] += props.deltaX;
        setWidths(newWidths);
    }

    const rowRenderer = (props: TableRowProps) => {
        if (props.index === selectedRow) {
            props.className += ' selected';
        }
        props.onRowClick = () => {
            setSelectedRow(props.index);
            if (props.index === editingRow) {
                return;
            }
            if (editingRow > -1) {
                // save
                toastSuccess();
                setEditingRow(-1);
            }
        };
        return defaultTableRowRenderer(props);
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

    return (
        <>
        <div style={{height: window.innerHeight - 140}}>
            <Table
            width={window.innerWidth - 20}
            height={window.innerHeight - 160}
            headerHeight={25}
            rowHeight={25}
            rowCount={songs.length}
            rowRenderer={rowRenderer}
            rowGetter={({ index }) => songs[index]}
        >
        <Column
            headerRenderer={headerRenderer}
            dataKey=''
            label=''
            cellRenderer={({rowIndex, dataKey})=> editCellRenderer(rowIndex)}
            width={widths.edit}
            />
        <Column
            headerRenderer={headerRenderer}
            dataKey='name'
            label='Title'
            cellRenderer={({rowIndex, dataKey})=> genericCellRenderer(rowIndex, 'name')}
            width={widths.name}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey='albumArtist'
            label='Album Artist'
            cellRenderer={({rowIndex})=> genericCellRenderer(rowIndex, 'albumArtist') }
            width={widths.albumArtist}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey='artist'
            label='Artist'
            cellRenderer={({rowIndex})=> genericCellRenderer(rowIndex, 'artist') }
            width={widths.artist}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey='album'
            label='Album'
            cellRenderer={({rowIndex})=> genericCellRenderer(rowIndex, 'album') }
            width={widths.album}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey='track'
            label='Track'
            cellRenderer={({rowIndex})=> trackRenderer(rowIndex) }
            width={widths.track}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey='path'
            label='Path'
            cellRenderer={({rowIndex}) => pathRenderer(rowIndex) }
            width={widths.path}
        />
        </Table>
        </div>
        <Controls isPlaying={isPlaying} setIsPlaying={setIsPlaying} onPause={onPause} onPlay={onPlay} onStop={onStop}/>
       </>
    );
}
