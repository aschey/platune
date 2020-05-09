import React, { useState, useEffect } from "react";
import { Column, Table, TableHeaderRenderer, TableHeaderProps } from "react-virtualized";
import Draggable from "react-draggable";
import { Song } from "../models/song";
import { range, sleep } from "../util";
import { getJson } from "../fetchUtil";
import _ from "lodash";
import { Cell } from "@blueprintjs/table";
import { Intent, EditableText, Text } from "@blueprintjs/core";
import { toastSuccess } from "../appToaster";
import { audioQueue } from "../audio";
import { Controls } from "./Controls";

export const Demo: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [height, setHeight] = useState(window.innerHeight - 39);
    const [playingRow, setPlayingRow] = useState(-1);
    const [selectedRow, setSelectedRow] = useState(-1);
    const [editingRow, setEditingRow] = useState(-1);
    const [isPlaying, setIsPlaying] = useState(false);
    const [widths, setWidths] = useState({
        name: 200,
        albumArtist: 200,
        artist: 200
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
        if (rowIndex === editingRow && canEdit) {
            return (
                <div className='bp3-table-cell bp3-intent-primary' style={{display: 'flex', flex: 1, height: 20}} onDoubleClick={() => onDoubleClick(rowIndex)}>
                    <EditableText value={value}/>
                </div>);
        }

        if (rowIndex === playingRow) {
            return (
                <div className='bp3-table-cell bp3-intent-success' style={{display: 'flex', flex: 1, height: 20}} onDoubleClick={() => onDoubleClick(rowIndex)}>
                    {value}
                </div>);
        }
        return (
            <div className='bp3-table-cell' style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a', display: 'flex', flex: 1, height: 20}} onDoubleClick={() => onDoubleClick(rowIndex)}>
                {value}
            </div>
        );
    }

    const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album') => {
        let value = songs[rowIndex][field].toString();
        return cellRenderer(rowIndex, value);
    }

    const resizeRow = (props: {dataKey: string, deltaX: number}) => {
        const newWidths: any =_.cloneDeep(widths);
        
        newWidths[props.dataKey] += props.deltaX;
        setWidths(newWidths);
    }

    const newRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album') => {
        return <Cell style={{backgroundColor: rowIndex % 2 == 0 ? '#334554' : '#2c3d4a', display: 'flex', flexGrow: 1, margin: 0}}>
            <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                {songs[rowIndex][field]}
            </div>
        </Cell>
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
            headerHeight={20}
            rowHeight={20}
            rowCount={songs.length}
            rowGetter={({ index }) => songs[index]}
        >
        <Column
            headerRenderer={headerRenderer}
            dataKey="name"
            label="Title"
            cellRenderer={({rowIndex, dataKey})=> genericCellRenderer(rowIndex, 'name')}
            width={widths.name}
        />
        <Column
            headerRenderer={headerRenderer}
            dataKey="albumArtist"
            label="Album Artist"
            cellRenderer={({rowIndex})=> genericCellRenderer(rowIndex, 'albumArtist') }
            width={widths.albumArtist}
        />
        <Column
            dataKey="artist"
            label="Artist"
            cellRenderer={({rowIndex})=> genericCellRenderer(rowIndex, 'artist') }
            width={widths.artist}
        />
        </Table>
        </div>
        <Controls isPlaying={isPlaying} setIsPlaying={setIsPlaying} onPause={onPause} onPlay={onPlay} onStop={onStop}/>
       </>
    );
}
