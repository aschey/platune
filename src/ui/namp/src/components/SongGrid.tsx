import React, { useState, useEffect } from "react";
import { Column, Table, TableHeaderRenderer, TableHeaderProps, defaultTableRowRenderer, TableRowProps, RowMouseEventHandlerParams, CellMeasurerCache, CellMeasurer } from "react-virtualized";
import Draggable from "react-draggable";
import { Song } from "../models/song";
import { range, sleep, formatMs } from "../util";
import { getJson } from "../fetchUtil";
import _, { Dictionary } from "lodash";
import { Intent, EditableText, Text, Button } from "@blueprintjs/core";
import { toastSuccess } from "../appToaster";
import { audioQueue } from "../audio";
import { Controls } from "./Controls";
import { FlexCol } from "./FlexCol";
import { FlexRow } from "./FlexRow";

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [groupedSongs, setGroupedSongs] = useState<Dictionary<Song[]>>({});
    const [albumKeys, setAlbumKeys] = useState<string[]>([]);
    const [playingRow, setPlayingRow] = useState(-1);
    const [playingMillis, setPlayingMillis] = useState(-1);
    const [progress, setProgress] = useState(-1);
    const [startTime, setStartTime] = useState(0);
    const [pauseTime, setPauseTime] = useState(0);
    const [pauseStart, setPauseStart] = useState(0);
    const [selectedRow, setSelectedRow] = useState(-1);
    const [editingRow, setEditingRow] = useState(-1);
    const [isPlaying, setIsPlaying] = useState(false);
    const [widths, setWidths] = useState({
        edit: 30,
        name: 300,
        albumArtist: 250,
        artist: 250,
        album: 250,
        track: 70,
        time: 60,
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
        loadSongs().then(s => {
            setSongs(s);
            let g = _.groupBy(s, ss => ss.albumArtist + " " + ss.album);
            setGroupedSongs(g);
            setAlbumKeys(_.keys(g));
        });
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

    useEffect(() => {
        if (playingRow === -1) {
            return;
        }
        const updateInterval = 60;
        setPlayingMillis(songs[playingRow].time);
        const interval = setInterval(() => {
            if (isPlaying) {
                setProgress(new Date().getTime() - pauseTime - startTime);
            }
        }, updateInterval);
        return () => clearTimeout(interval);
    }, [playingRow, isPlaying, pauseTime, songs, startTime]);


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
        updatePlayingRow(songIndex);
        audioQueue.stop();
        startQueue(songIndex);
    }

    const startQueue = (songIndex: number) => {
        return audioQueue.start([
            songs[songIndex].path,
            songs[songIndex + 1].path
        ], songIndex, onSongFinished);
    }

    const updatePlayingRow = (rowIndex: number) => {
        setPauseTime(0);
        setStartTime(new Date().getTime());
        setPlayingRow(rowIndex);
    }

    const onSongFinished = (playingRow: number) => {
        updatePlayingRow(playingRow + 1);
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

    const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album' | 'time') => {
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

    const timeRenderer = (rowIndex: number) => {
        let value = songs[rowIndex]['time'];
        let fmtValue = formatMs(value);
        return cellRenderer(rowIndex, fmtValue);
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
        setPauseStart(new Date().getTime());
    }

    const onPlay = () => {
        const rowToPlay = playingRow > -1 ? playingRow : selectedRow;
        if (pauseStart > 0) {
            setPauseTime(prev => prev + (new Date().getTime() - pauseStart));
            setPauseStart(0);
        }
        else {
            updatePlayingRow(rowToPlay);
        }
        startQueue(rowToPlay);
    }

    const onStop = () => {
        audioQueue.stop();
        setPauseStart(0);
        updatePlayingRow(-1);
    }

    const cache = new CellMeasurerCache({
        defaultWidth: 150,
        fixedWidth: true,
    });

    const otherGrid =
        <div style={{height: window.innerHeight - 140}}>
            <Table
            width={window.innerWidth - 20}
            height={window.innerHeight - 160}
            headerHeight={25}
            rowCount={albumKeys.length}
            rowRenderer={rowRenderer}
            rowHeight={cache.rowHeight}
            rowGetter={({ index }) => groupedSongs[albumKeys[index]]}
            >
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='album'
                    label='Album'
                    cellRenderer={({rowIndex, dataKey})=> {
                        let g = groupedSongs[albumKeys[rowIndex]][0];
                        return <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                            <FlexCol>
                                <div>{g.artist}</div>
                                <div>{g.album}</div>
                            </FlexCol>
                        
                    </div>
                    }}
                    width={widths.album}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='name'
                    label='Title'
                    cellRenderer={({rowIndex, dataKey, parent})=> {
                        let g = groupedSongs[albumKeys[rowIndex]];
                        return <CellMeasurer
                            cache={cache}
                            columnIndex={1}
                            key={dataKey}
                            parent={parent}
                            rowIndex={rowIndex}>
                            <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                            <FlexCol>
                                {g.map(gg => <div>{gg.name}</div>)}
                            </FlexCol>
                        
                            </div>
                            </CellMeasurer>
                    }}
                    width={widths.name}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='time'
                    label='Time'
                    cellRenderer={({rowIndex, dataKey, parent})=> {
                        let g = groupedSongs[albumKeys[rowIndex]];
                        return <CellMeasurer
                            cache={cache}
                            columnIndex={2}
                            key={dataKey}
                            parent={parent}
                            rowIndex={rowIndex}>
                            <div onDoubleClick={() => onDoubleClick(rowIndex)}>
                            <FlexCol>
                                {g.map(gg => <div>{gg.time}</div>)}
                            </FlexCol>
                        
                            </div>
                            </CellMeasurer>
                    }}
                    width={widths.time}
                />
            </Table>
        </div>

    const mainGrid = 
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
                dataKey='time'
                label='Time'
                cellRenderer={({rowIndex})=> timeRenderer(rowIndex) }
                width={widths.time}
            />
            <Column
                headerRenderer={headerRenderer}
                dataKey='path'
                label='Path'
                cellRenderer={({rowIndex}) => pathRenderer(rowIndex) }
                width={widths.path}
            />
        </Table>
    </div>;

    return (
        <>
        {otherGrid}
        <Controls 
        isPlaying={isPlaying} 
        setIsPlaying={setIsPlaying} 
        onPause={onPause} 
        onPlay={onPlay} 
        onStop={onStop} 
        songMillis={playingMillis} 
        progress={progress}
    />
    </>
    );
}
