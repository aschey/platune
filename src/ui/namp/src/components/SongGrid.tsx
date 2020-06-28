import React, { useState, useEffect } from "react";
import { Column, Table, TableHeaderRenderer, TableHeaderProps, defaultTableRowRenderer, TableRowProps, RowMouseEventHandlerParams, CellMeasurerCache, CellMeasurer } from "react-virtualized";
import Draggable from "react-draggable";
import { Song } from "../models/song";
import { range, sleep, formatMs, formatRgb, setCssVar, formatRgba } from "../util";
import { getJson } from "../fetchUtil";
import _, { Dictionary } from "lodash";
import { Intent, EditableText, Text, Button } from "@blueprintjs/core";
import { toastSuccess } from "../appToaster";
import { audioQueue } from "../audio";
import { Controls } from "./Controls";
import { FlexCol } from "./FlexCol";
import { FlexRow } from "./FlexRow";
import { getProcessMemoryInfo } from "process";
import { Rgb } from "../models/rgb";

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
    const [selectedAlbumRow, setSelectedAlbumRow] = useState(-1);
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
    const [widths2, setWidths2] = useState({
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

    const loadColors = async (songId: number) => {
        const colors = await getJson<Rgb[]>(`/albumArtColors?songId=${songId}`);
        return colors;
    }

    useEffect(() => {
        loadSongs().then(s => {
            s.forEach((song, i) => song.index = i);
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
                    bounds={{right: 10, left: 0, top: 0, bottom: 0}}
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
        const isPlayingRow = playingRow === rowIndex;
        return (
        <div className='bp3-table-cell gridCell striped' style={{padding: 0, borderLeft: 'rgba(16, 22, 26, 0.4) 1px solid'}} key={rowIndex}>
            <FlexCol>
                <Button small minimal className={rowIndex === playingRow ? 'playing' : ''} icon={isEditingRow ?  'saved' : isPlayingRow ? 'volume-up' : 'edit'} onClick={() => {
                    const cur = songs[rowIndex];
                    let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
                    updateColors(cur.id, albumIndex);
                    if (isEditingRow) {
                        // save
                        toastSuccess();
                        setEditingRow(-1);
                    }
                    else {
                        setSelectedRow(rowIndex);
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
                <div key={rowIndex} className='bp3-table-cell selected gridCell editing' 
                  onDoubleClick={() => onDoubleClick(rowIndex)}
                  onClick={() => onRowClick(rowIndex)}>
                    { canEdit ? <EditableText defaultValue={value} className='editing'/> : value }
                </div>);
        }

        if (rowIndex === playingRow) {
            return (
                <div key={rowIndex} className='bp3-table-cell playing gridCell'
                  onDoubleClick={() => onDoubleClick(rowIndex)}
                  onClick={() => onRowClick(rowIndex)}>
                    {value}
                </div>);
        }
        if (rowIndex === selectedRow) {
            return (
                <div key={rowIndex} className='bp3-table-cell gridCell selected' 
                  onDoubleClick={() => onDoubleClick(rowIndex)}
                  onClick={() => onRowClick(rowIndex)}>
                    {value}
                </div>
            );
        }
        return (
            <div key={rowIndex} className='bp3-table-cell striped gridCell' 
              onDoubleClick={() => onDoubleClick(rowIndex)}
              onClick={() => onRowClick(rowIndex)}>
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

    const onRowClick = (index: number) => {
        setSelectedRow(index);
        const cur = songs[index];
        let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
        updateColors(cur.id, albumIndex);

        if (index === editingRow) {
            return;
        }
        if (editingRow > -1) {
            // save
            toastSuccess();
            setEditingRow(-1);
        }
    }

    const updateColors = async (songIndex: number, albumIndex: number) => {
        const colors = await loadColors(songIndex);
        const bg = colors[0];
        const fg = colors[1];
        const secondary = colors[2];
        setCssVar('--text-color', formatRgb(fg));
        setCssVar('--bg-1', formatRgba(bg, 0.2));
        setCssVar('--bg-2', formatRgba(bg, 0.4));
        setCssVar('--stripe', formatRgba(bg, 0.7));
        setCssVar('--selected-bg', formatRgba(secondary, 0.6));
        setCssVar('--playing-bg', formatRgba(colors[3], 0.6));
        setCssVar('--editing-color', formatRgb(colors[4]));
        setSelectedAlbumRow(albumIndex);
    }

    const rowRenderer = (props: TableRowProps) => {
        props.className += ' row';
        return defaultTableRowRenderer(props);
    }

    const rowRenderer2 = (props: TableRowProps) => {
        props.style.border = '1px solid rgb(38, 53, 64)';
        props.style.boxShadow = 'rgb(38, 53, 64) 1px 1px';
        props.style.background = 'rgb(51, 70, 84)';
        props.style.borderRadius = 5;
        props.style.transition = 'var(--transition)';
        props.style.left = 10;
        if (props.index === selectedAlbumRow) {
            props.className += ' selectedRow';
        }
        
        props.style.height -= 15;
        props.style.width -= 30;
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

    const mulitSongRenderer = (rowIndex: number, cellRenderer: (index: number) => void) => {
        let g = groupedSongs[albumKeys[rowIndex]];
        return <div className='rowParent'>
                    {g.map(gg => cellRenderer(gg.index))}
                </div>
    }

    const otherGrid =
        <div style={{height: window.innerHeight - 140}}>
            <Table
            width={window.innerWidth - 20}
            height={window.innerHeight - 160}
            headerHeight={25}
            rowCount={albumKeys.length}
            rowRenderer={rowRenderer2}
            overscanRowCount={0}
            estimatedRowSize={groupedSongs?.keys?.length > 0 ? songs.length / groupedSongs.keys.length * 25 : 250}
            rowHeight={index => Math.max(groupedSongs[albumKeys[index.index]].length * 25 + 20, 145)}
            rowGetter={({ index }) => groupedSongs[albumKeys[index]]}
            >
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='album'
                    label='Album'
                    cellRenderer={({rowIndex, dataKey, parent})=> {
                        let gg = groupedSongs[albumKeys[rowIndex]];
                        let g = groupedSongs[albumKeys[rowIndex]][0];
                        return <FlexCol 
                            style={{paddingTop: 5, height: Math.max(gg.length * 25, 125)}} 
                            onClick = {() => updateColors(g.id, rowIndex)}
                            >
                                <div>{g.artist}</div>
                                <div>{g.album}</div>
                                {g.hasArt ? 
                                    <img loading='lazy' src={`http://localhost:5000/albumArt?songId=${g.id}`} width={75} height={75} />
                                    : null }
                                
                    </FlexCol>
                        
            
                    }}
                    width={widths.album}
                />
                <Column
                  headerRenderer={headerRenderer}
                  dataKey=''
                  label=''
                  cellRenderer={({rowIndex}) => mulitSongRenderer(rowIndex, editCellRenderer)}
                  width={widths.edit}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='name'
                    label='Title'
                    cellRenderer={({rowIndex, dataKey, parent}) => 
                        mulitSongRenderer(rowIndex, i => genericCellRenderer(i, 'name'))}
                    width={widths.name}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='track'
                    label='Track'
                    cellRenderer={({rowIndex})=> mulitSongRenderer(rowIndex, trackRenderer)}
                    width={widths.track}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='time'
                    label='Time'
                    cellRenderer={({rowIndex}) => mulitSongRenderer(rowIndex, timeRenderer)}
                    width={widths.time}
                />
                <Column
                    headerRenderer={headerRenderer}
                    dataKey='path'
                    label='Path'
                    cellRenderer={({rowIndex}) => mulitSongRenderer(rowIndex, pathRenderer)}
                    width={widths.path}
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
