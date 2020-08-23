import React, { useState, useEffect } from 'react';
import {
  Column,
  Table,
  TableHeaderRenderer,
  TableHeaderProps,
  defaultTableRowRenderer,
  TableRowProps,
  RowMouseEventHandlerParams,
  CellMeasurerCache,
  CellMeasurer,
} from 'react-virtualized';
import Draggable from 'react-draggable';
import { Song } from '../models/song';
import { range, sleep, formatMs, formatRgb, setCssVar } from '../util';
import { getJson } from '../fetchUtil';
import _, { Dictionary } from 'lodash';
import { Intent, EditableText, Text, Button, Icon } from '@blueprintjs/core';
import { toastSuccess } from '../appToaster';
import { audioQueue } from '../audio';
import { Controls } from './Controls';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { getProcessMemoryInfo } from 'process';
import { Rgb } from '../models/rgb';
import { useObservable } from 'rxjs-hooks';

interface SongGridProps {
  selectedGrid: string;
  isLightTheme: boolean;
  width: number;
  songs: Song[];
  setSongs: (songs: Song[]) => void;
  queuedSongs: Song[];
  setQueuedSongs: (songs: Song[]) => void;
  queuePlayingFile: string;
  setQueuePlayingFile: (playingFile: string) => void;
}

export const SongGrid: React.FC<SongGridProps> = ({
  selectedGrid,
  isLightTheme,
  width,
  songs,
  setSongs,
  queuedSongs,
  setQueuedSongs,
  queuePlayingFile,
  setQueuePlayingFile,
}) => {
  const [groupedSongs, setGroupedSongs] = useState<Dictionary<Song[]>>({});
  const [albumKeys, setAlbumKeys] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState('');
  const [selectedAlbum, setSelectedAlbum] = useState('');
  const [editingFile, setEditingFile] = useState('');
  const [isPlaying, setIsPlaying] = useState(false);
  const [playingFile, setPlayingFile] = useState('');
  const [widths, setWidths] = useState({
    edit: 30,
    name: 300,
    albumArtist: 250,
    artist: 250,
    album: 250,
    track: 70,
    time: 60,
    path: 300,
  });
  const [widths2, setWidths2] = useState({
    edit: 30,
    name: 300,
    albumArtist: 250,
    artist: 250,
    album: 250,
    track: 70,
    time: 60,
    path: 400,
  });

  const mainRef = React.createRef<Table>();
  const otherRef = React.createRef<Table>();
  const songFinishedIndex = useObservable(() => audioQueue.onEnded);
  const numTries = 10;

  const loadSongs = async () => {
    for (let i of range(numTries)) {
      try {
        const songs = await getJson<Song[]>('/songs?offset=0&limit=15000');
        return songs;
      } catch (e) {
        if (i === numTries - 1) {
          throw e;
        }
        await sleep(1000);
      }
    }
    return [];
  };

  const loadColors = async (songId: number) => {
    const colors = await getJson<Rgb[]>(`/albumArtColors?songId=${songId}&isLight=${isLightTheme}`);
    return colors;
  };

  useEffect(() => {
    loadSongs().then(setSongs);
  }, []);

  useEffect(() => {
    if (songFinishedIndex !== null) {
      onSongFinished(songFinishedIndex.path);
    }
  }, [songFinishedIndex]);

  useEffect(() => {
    //onStop();
    songs.forEach((song, i) => (song.index = i));
    let g = _.groupBy(songs, ss => ss.albumArtist + ' ' + ss.album);
    setGroupedSongs(g);
    setAlbumKeys(_.keys(g));
  }, [songs]);

  useEffect(() => {
    if (songs.length === 0) {
      return;
    }
    const ref = selectedGrid === 'song' ? mainRef.current : otherRef.current;
    ref?.forceUpdate();
    ref?.forceUpdateGrid();
    ref?.measureAllRows();
    ref?.recomputeRowHeights();
  }, [width, selectedGrid, songs, mainRef, otherRef]);

  useEffect(() => {
    setIsPlaying(playingFile !== '');
  }, [playingFile]);

  useEffect(() => {
    if (selectedGrid === 'song') {
      mainRef.current?.recomputeRowHeights();
      setCssVar('--header-padding', '5px');
    } else {
      otherRef.current?.recomputeRowHeights();
      setCssVar('--header-padding', '16px');
    }
  }, [selectedGrid]);

  const headerRenderer = (props: TableHeaderProps) => {
    return (
      <>
        <div className='ReactVirtualized__Table__headerTruncatedText'>{props.label}</div>
        <Draggable
          axis='none'
          defaultClassName='DragHandle'
          defaultClassNameDragging='DragHandleActive'
          //bounds={{right: 100, left: 100, top: 0, bottom: 0}}
          onDrag={(event, { deltaX }) => {
            resizeRow({ dataKey: props.dataKey, deltaX });
          }}
        >
          <span className='DragHandleIcon'>⋮</span>
        </Draggable>
      </>
    );
  };

  const editCellRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const isEditingRow = editingFile === path;
    const isSelectedRow = selectedFile === path;
    const isPlayingRow = playingFile === path;
    const classes = `${isEditingRow ? 'editing' : ''} ${
      isPlayingRow ? 'playing' : isSelectedRow ? 'selected' : 'striped'
    }`;
    return (
      <div
        className={`bp3-table-cell grid-cell ${classes}`}
        style={{ padding: 0, borderLeft: 'rgba(16, 22, 26, 0.4) 1px solid' }}
        key={rowIndex}
      >
        <FlexCol>
          <Button
            small
            minimal
            className={isPlayingRow ? 'playing' : ''}
            icon={isEditingRow ? 'saved' : isPlayingRow ? 'volume-up' : 'edit'}
            onClick={() => {
              const cur = songs[rowIndex];
              let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
              if (selectedGrid === 'album') {
                updateSelectedAlbum(cur.id, albumIndex);
              }

              if (isEditingRow) {
                // save
                toastSuccess();
                setEditingFile('');
              } else {
                setSelectedFile(path);
                setEditingFile(path);
              }
            }}
          />
        </FlexCol>
      </div>
    );
  };

  const onDoubleClick = (path: string) => {
    if (path === editingFile) {
      return;
    }
    if (editingFile !== '') {
      // save
      toastSuccess();
      setEditingFile('');
    }
    updatePlayingPath(path);
    audioQueue.stop();
    startQueue(path);
  };

  const startQueue = (path: string) => {
    const index = songs.map(s => s.path).indexOf(path);
    const queue = songs.filter(s => s.index >= index);
    setQueuedSongs(queue);
    setQueuePlayingFile(queue[0].path);
    return audioQueue.start(queue.map(q => q.path));
  };

  const updatePlayingPath = (path: string) => {
    setPlayingFile(path);
    setQueuePlayingFile(path);
  };

  const onSongFinished = (playingPath: string) => {
    const next = getNext(queuedSongs, playingPath);
    if (next !== '') {
      updatePlayingPath(next);
    }
  };

  const getNext = (songs: Song[], current: string) => {
    const currentIndex = songs.map(s => s.path).indexOf(current);
    if (currentIndex === songs.length - 1) {
      return '';
    }
    return songs[currentIndex + 1].path;
  };

  const cellRenderer = (path: string, value: string, canEdit: boolean = true) => {
    let classes = 'bp3-table-cell grid-cell';
    let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (path === selectedFile) {
      classes += ' selected';
    } else {
      classes += ' striped';
    }
    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={() => onRowClick(path)}>
        <div className='ellipsize' style={{ display: 'inline-block' }}>
          {child}
        </div>
      </div>
    );
  };

  const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album' | 'time') => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const value = songs[rowIndex][field].toString();
    return cellRenderer(path, value);
  };

  const trackRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex].track.toString();
    if (value === '0') {
      value = '';
    }
    return cellRenderer(path, value);
  };

  const timeRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex]['time'];
    let fmtValue = formatMs(value);
    return cellRenderer(path, fmtValue);
  };

  const pathRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex].path;
    return cellRenderer(path, value, false);
  };

  const resizeRow = (props: { dataKey: string; deltaX: number }) => {
    const newWidths: any = _.cloneDeep(selectedGrid === 'song' ? widths : widths2);

    newWidths[props.dataKey] += props.deltaX;
    if (selectedGrid === 'song') {
      setWidths(newWidths);
    } else {
      setWidths2(newWidths);
    }
  };

  const onRowClick = (path: string) => {
    setSelectedFile(path);
    const cur = songs.filter(s => s.path === path)[0];
    let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
    updateSelectedAlbum(cur.id, albumIndex);

    if (path === editingFile) {
      return;
    }
    if (editingFile !== '') {
      // save
      toastSuccess();
      setEditingFile('');
    }
  };

  const getAlbumSongs = (albumIndex: number) => groupedSongs[albumKeys[albumIndex]];

  const updateSelectedAlbum = async (songIndex: number, albumIndex: number) => {
    if (getAlbumSongs(albumIndex)[0].hasArt) {
      updateColors(songIndex, albumIndex);
    }
    setSelectedAlbum(albumKeys[albumIndex]);
  };

  const updateColors = async (songIndex: number, albumIndex: number) => {
    const colors = await loadColors(songIndex);
    const bg = colors[0];
    const fg = colors[1];
    const secondary = colors[2];
    setCssVar('--grid-selected-text-color', formatRgb(fg));
    setCssVar('--grid-selected-shadow-1', formatRgb(bg));
    setCssVar('--grid-selected-shadow-2', formatRgb(bg));
    setCssVar('--grid-selected-stripe-even', formatRgb(bg));
    setCssVar('--grid-selected-background', formatRgb(secondary));
    setCssVar('--grid-selected-playing-row-background', formatRgb(colors[3]));
    setCssVar('--grid-selected-editing-row-color', formatRgb(colors[4]));
  };

  const rowRenderer = (props: TableRowProps) => {
    props.className += ' row';
    return defaultTableRowRenderer(props);
  };

  const rowRenderer2 = (props: TableRowProps) => {
    props.className += ' card';
    props.style.left = 10;
    if (props.index === albumKeys.indexOf(selectedAlbum)) {
      props.className += ' album-selected-row';
    }
    if (groupedSongs[albumKeys[props.index]][0].hasArt) {
      props.className += ' has-art';
    }

    props.style.height -= 15;
    props.style.width -= 30;
    return defaultTableRowRenderer(props);
  };

  const onPause = async () => {
    await audioQueue.pause();
    setIsPlaying(false);
  };

  const onPlay = () => {
    const fileToPlay = playingFile !== '' ? playingFile : selectedFile;
    updatePlayingPath(fileToPlay);
    startQueue(fileToPlay);
  };

  const onStop = () => {
    audioQueue.stop();
    updatePlayingPath('');
  };

  const multiSongRenderer = (rowIndex: number, cellRenderer: (index: number) => void) => {
    let g = groupedSongs[albumKeys[rowIndex]];
    return <div className='rowParent'>{g.map(gg => cellRenderer(gg.index))}</div>;
  };

  const otherGrid = (
    <div style={{ height: window.innerHeight - 110 }}>
      <Table
        ref={otherRef}
        width={width - 5}
        height={window.innerHeight - 110}
        headerHeight={25}
        rowCount={albumKeys.length}
        rowRenderer={rowRenderer2}
        overscanRowCount={0}
        estimatedRowSize={groupedSongs?.keys?.length > 0 ? (songs.length / groupedSongs.keys.length) * 25 : 250}
        rowHeight={index => Math.max(groupedSongs[albumKeys[index.index]].length * 25 + 40, 145)}
        rowGetter={({ index }) => groupedSongs[albumKeys[index]]}
      >
        <Column
          headerRenderer={headerRenderer}
          dataKey='album'
          label='Album'
          cellRenderer={({ rowIndex }) => {
            let gg = groupedSongs[albumKeys[rowIndex]];
            let g = groupedSongs[albumKeys[rowIndex]][0];
            return (
              <FlexCol
                style={{ paddingLeft: 10, height: Math.max(gg.length * 25, 125) }}
                onClick={() => updateSelectedAlbum(g.id, rowIndex)}
              >
                <div>{g.albumArtist}</div>
                <div style={{ paddingBottom: 5 }}>{g.album}</div>
                {g.hasArt ? (
                  <img loading='lazy' src={`http://localhost:5000/albumArt?songId=${g.id}`} width={75} height={75} />
                ) : null}
              </FlexCol>
            );
          }}
          width={widths2.album}
          minWidth={widths2.album}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey=''
          label=''
          cellRenderer={({ rowIndex }) => multiSongRenderer(rowIndex, editCellRenderer)}
          width={widths2.edit}
          minWidth={widths2.edit}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='name'
          label='Title'
          cellRenderer={({ rowIndex, dataKey }) => multiSongRenderer(rowIndex, i => genericCellRenderer(i, 'name'))}
          width={widths2.name}
          minWidth={widths2.name}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='track'
          label='Track'
          cellRenderer={({ rowIndex }) => multiSongRenderer(rowIndex, trackRenderer)}
          width={widths2.track}
          minWidth={widths2.track}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='time'
          label='Time'
          cellRenderer={({ rowIndex }) => multiSongRenderer(rowIndex, timeRenderer)}
          width={widths2.time}
          minWidth={widths2.time}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='path'
          label='Path'
          cellRenderer={({ rowIndex }) => multiSongRenderer(rowIndex, pathRenderer)}
          width={widths2.path}
          minWidth={widths2.path}
        />
      </Table>
    </div>
  );

  const mainGrid = (
    <div style={{ height: window.innerHeight - 110 }}>
      <Table
        ref={mainRef}
        width={width - 5}
        height={window.innerHeight - 110}
        headerHeight={25}
        rowHeight={25}
        rowCount={songs.length}
        rowRenderer={rowRenderer}
        rowGetter={({ index }) => songs[index]}
      >
        <Column
          headerRenderer={headerRenderer}
          dataKey=''
          cellRenderer={({ rowIndex }) => editCellRenderer(rowIndex)}
          width={widths.edit}
          minWidth={widths.edit}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='name'
          label='Title'
          cellRenderer={({ rowIndex }) => genericCellRenderer(rowIndex, 'name')}
          width={widths.name}
          minWidth={widths.name}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='albumArtist'
          label='Album Artist'
          cellRenderer={({ rowIndex }) => genericCellRenderer(rowIndex, 'albumArtist')}
          width={widths.albumArtist}
          minWidth={widths.albumArtist}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='artist'
          label='Artist'
          cellRenderer={({ rowIndex }) => genericCellRenderer(rowIndex, 'artist')}
          width={widths.artist}
          minWidth={widths.artist}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='album'
          label='Album'
          cellRenderer={({ rowIndex }) => genericCellRenderer(rowIndex, 'album')}
          width={widths.album}
          minWidth={widths.album}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='track'
          label='Track'
          cellRenderer={({ rowIndex }) => trackRenderer(rowIndex)}
          width={widths.track}
          minWidth={widths.track}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='time'
          label='Time'
          cellRenderer={({ rowIndex }) => timeRenderer(rowIndex)}
          width={widths.time}
          minWidth={widths.time}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='path'
          label='Path'
          cellRenderer={({ rowIndex }) => pathRenderer(rowIndex)}
          width={widths.path}
          minWidth={widths.path}
        />
      </Table>
    </div>
  );

  return (
    <>
      <div>{selectedGrid === 'song' ? mainGrid : otherGrid}</div>

      <Controls
        isPlaying={isPlaying}
        setIsPlaying={setIsPlaying}
        onPause={onPause}
        onPlay={onPlay}
        onStop={onStop}
        playingSong={playingFile !== '' ? queuedSongs.filter(s => s.path === playingFile)[0] : null}
      />
    </>
  );
};
