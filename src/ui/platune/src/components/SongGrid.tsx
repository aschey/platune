import { Button, EditableText, Text, Tag, Intent, Colors } from '@blueprintjs/core';
import _, { Dictionary } from 'lodash';
import React, { useCallback, useEffect, useState, useReducer, useRef } from 'react';
import { default as DraggableCol } from 'react-draggable';
import {
  Draggable,
  DraggableProvided,
  DraggableStateSnapshot,
  Droppable,
  DraggableRubric,
  DroppableProvided,
  DroppableStateSnapshot,
} from 'react-beautiful-dnd';
import {
  Column,
  defaultTableRowRenderer,
  GridCellProps,
  Table,
  TableCellProps,
  TableHeaderProps,
  TableRowProps,
} from 'react-virtualized';
import { useObservable } from 'rxjs-hooks';
import { toastSuccess } from '../appToaster';
import { audioQueue } from '../audio';
import { getJson } from '../fetchUtil';
import { Rgb } from '../models/rgb';
import { Song } from '../models/song';
import { formatMs, formatRgb, range, setCssVar, sleep } from '../util';
import { Controls } from './Controls';
import { FlexCol } from './FlexCol';
import { normal } from 'color-blend';
import { hexToRgb } from '../themes/colorMixer';
import { lightTheme } from '../themes/light';
import ReactDOM from 'react-dom';
import { FlexRow } from './FlexRow';
import { GridTag } from './GridTag';
import { theme } from './App';

interface SongGridProps {
  selectedGrid: string;
  isLightTheme: boolean;
  width: number;
  height: number;
  songs: Song[];
  setSongs: (songs: Song[]) => void;
  queuedSongs: Song[];
  setQueuedSongs: (songs: Song[]) => void;
  selectedFiles: string[];
  setSelectedFiles: (selectedFiles: string[]) => void;
}

export const SongGrid: React.FC<SongGridProps> = ({
  selectedGrid,
  isLightTheme,
  width,
  height,
  songs,
  setSongs,
  queuedSongs,
  setQueuedSongs,
  selectedFiles,
  setSelectedFiles,
}) => {
  const [groupedSongs, setGroupedSongs] = useState<Dictionary<Song[]>>({});
  const [albumKeys, setAlbumKeys] = useState<string[]>([]);

  const [selectedAlbum, setSelectedAlbum] = useState('');
  const [editingFile, setEditingFile] = useState('');

  const editWidth = 30;
  const trackWidth = 70;
  const timeWidth = 60;
  const remainingWidth = width - editWidth - trackWidth - timeWidth;
  const [widths, setWidths] = useState({
    edit: editWidth,
    name: remainingWidth * 0.2,
    albumArtist: remainingWidth * 0.2,
    artist: remainingWidth * 0.2,
    album: remainingWidth * 0.2,
    track: trackWidth,
    time: timeWidth,
    tags: remainingWidth * 0.2,
  });
  const [widths2, setWidths2] = useState({
    edit: editWidth,
    name: remainingWidth * 0.4,
    albumArtist: remainingWidth * 0,
    artist: remainingWidth * 0,
    album: remainingWidth * 0.2,
    track: trackWidth,
    time: timeWidth,
    tags: remainingWidth * 0.4,
  });

  const mainRef = useRef<Table>();
  const otherRef = useRef<Table>();
  const draggingFile = useRef<string>('');
  const playingFile = useObservable(() => audioQueue.playingSource);
  const numTries = 10;

  const loadSongs = useCallback(async () => {
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
  }, []);

  const loadColors = async (songId: number) => {
    const colors = await getJson<Rgb[]>(`/albumArtColors?songId=${songId}&isLight=${isLightTheme}`);
    return colors;
  };

  useEffect(() => {
    loadSongs().then(setSongs);
  }, [loadSongs, setSongs]);

  useEffect(() => {
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
    if (!ref?.props?.rowCount) {
      return;
    }
    ref.forceUpdate();
    ref.forceUpdateGrid();
    ref.measureAllRows();
    ref.recomputeRowHeights();
  }, [width, selectedGrid, songs, mainRef, otherRef]);

  useEffect(() => {
    if (selectedGrid === 'song') {
      mainRef.current?.recomputeRowHeights();
      setCssVar('--header-padding', '5px');
    } else {
      otherRef.current?.recomputeRowHeights();
      setCssVar('--header-padding', '16px');
    }
  }, [selectedGrid, mainRef, otherRef]);

  useEffect(() => {
    setWidths({
      edit: editWidth,
      name: remainingWidth * 0.2,
      albumArtist: remainingWidth * 0.2,
      artist: remainingWidth * 0.2,
      album: remainingWidth * 0.2,
      track: trackWidth,
      time: timeWidth,
      tags: remainingWidth * 0.2,
    });

    setWidths2({
      edit: editWidth,
      name: remainingWidth * 0.4,
      albumArtist: remainingWidth * 0,
      artist: remainingWidth * 0,
      album: remainingWidth * 0.2,
      track: trackWidth,
      time: timeWidth,
      tags: remainingWidth * 0.4,
    });
  }, [width, remainingWidth]);

  const headerRenderer = (props: TableHeaderProps) => {
    return (
      <>
        <div className='ReactVirtualized__Table__headerTruncatedText'>{props.label}</div>
        <DraggableCol
          axis='none'
          defaultClassName='DragHandle'
          defaultClassNameDragging='DragHandleActive'
          //bounds={{right: 100, left: 100, top: 0, bottom: 0}}
          onDrag={(event, { deltaX }) => {
            resizeRow({ dataKey: props.dataKey, deltaX });
          }}
        >
          <span className='DragHandleIcon'>⋮</span>
        </DraggableCol>
      </>
    );
  };

  const editCellRenderer = ({ rowIndex }: TableCellProps) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const isEditingRow = editingFile === path;
    const isSelectedRow = selectedFiles.indexOf(path) > -1;
    const isPlayingRow = playingFile === path;
    const classes = `${isEditingRow ? 'editing' : ''} ${
      isPlayingRow ? 'playing' : isSelectedRow ? 'selected' : rowIndex % 2 === 0 ? 'striped-even' : 'striped-odd'
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
            onClick={(e: React.MouseEvent) => {
              const cur = songs[rowIndex];
              let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
              if (selectedGrid === 'album') {
                const hasArt = getAlbumSongs(albumIndex).filter(s => s.hasArt);
                const song = hasArt.length > 0 ? hasArt[0] : cur;
                updateSelectedAlbum(song.id, song.hasArt, albumIndex);
              }

              if (isEditingRow) {
                // save
                toastSuccess();
                setEditingFile('');
              } else {
                onFileSelect(e, path);
                setEditingFile(path);
              }
            }}
          />
        </FlexCol>
      </div>
    );
  };

  const onFileSelect = (e: React.MouseEvent, path: string) => {
    if (e.ctrlKey) {
      setSelectedFiles(selectedFiles.concat([path]));
    } else if (e.shiftKey) {
      const paths = songs.map(s => s.path);
      const index = paths.indexOf(path);

      for (let i = index - 1; i >= 0; i--) {
        if (selectedFiles.indexOf(paths[i]) > -1) {
          setSelectedFiles(selectedFiles.concat(paths.slice(i + 1, index + 1)));
          break;
        }
      }
    } else {
      setSelectedFiles([path]);
    }
  };

  const onDoubleClick = async (path: string) => {
    if (path === editingFile) {
      return;
    }
    if (editingFile !== '') {
      // save
      toastSuccess();
      setEditingFile('');
    }
    await startQueue(path);
  };

  const startQueue = async (path: string) => {
    const index = songs.map(s => s.path).indexOf(path);
    const queue = songs.filter(s => s.index >= index);
    // Don't reset queue if currently paused and we're resuming the same song
    if (!(audioQueue.isPaused() && path === playingFile)) {
      setQueuedSongs(queue);
      audioQueue.setQueue(queue.map(q => q.path));
    }

    await audioQueue.start(queue[0].path);
  };

  const onPrevious = async () => {
    if (!playingFile) {
      return;
    }
    const playingIndex = audioQueue.queuedSongs.value.indexOf(playingFile);
    if (playingIndex > 0) {
      audioQueue.previous();
    } else {
      const songPaths = songs.map(s => s.path);
      const songIndex = songPaths.indexOf(playingFile);
      if (songIndex > 0) {
        await startQueue(songPaths[songIndex - 1]);
      }
    }
  };

  const cellRenderer = (rowIndex: number, path: string, value: string, canEdit: boolean = true) => {
    let classes = 'bp3-table-cell grid-cell';
    let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (selectedFiles.indexOf(path) > -1) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }

    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={e => onRowClick(e, path)}>
        <Text ellipsize>{child}</Text>
      </div>
    );
  };

  const cellRenderer2 = (
    rowIndex: number,
    path: string,
    value: string,
    draggingSong: string,
    canEdit: boolean = true
  ) => {
    let classes = 'bp3-table-cell grid-cell';
    let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (selectedFiles.indexOf(path) > -1) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }

    return path === draggingSong ? (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={e => onRowClick(e, path)}>
        <Text ellipsize>{child}</Text>
      </div>
    ) : (
      <Draggable draggableId={path} index={rowIndex} key={path}>
        {(provided: DraggableProvided, snapshot: DraggableStateSnapshot) => {
          return (
            <div
              ref={provided.innerRef}
              {...provided.dragHandleProps}
              {...provided.draggableProps}
              style={{ ...provided.draggableProps.style, ...provided.draggableProps.style, transform: 'none' }}
              key={path}
              className={classes}
              onDoubleClick={() => onDoubleClick(path)}
              onClick={e => onRowClick(e, path)}
            >
              <Text ellipsize>{child}</Text>
            </div>
          );
        }}
      </Draggable>
    );
  };

  const genericCellRenderer = ({ rowIndex, dataKey }: TableCellProps) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const value = (songs as any)[rowIndex][dataKey].toString();
    return cellRenderer(rowIndex, path, value);
  };

  const genericCellRenderer2 = (
    rowIndex: number,
    field: 'name' | 'albumArtist' | 'artist' | 'album' | 'time',
    draggingSong: string
  ) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const value = songs[rowIndex][field].toString();
    return cellRenderer2(rowIndex, path, value, draggingSong);
  };

  const trackRenderer = ({ rowIndex }: TableCellProps) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex].track.toString();
    if (value === '0') {
      value = '';
    }
    return cellRenderer(rowIndex, path, value);
  };

  const timeRenderer = ({ rowIndex }: TableCellProps) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex]['time'];
    let fmtValue = formatMs(value);
    return cellRenderer(rowIndex, path, fmtValue);
  };

  const tagRenderer = ({ rowIndex }: TableCellProps) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;

    let classes = 'bp3-table-cell grid-cell';
    //let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      //child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (selectedFiles.indexOf(path) > -1) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }

    const estWidth = songs[rowIndex].tags.reduce((prev, current) => prev + current.name.length * 6 + 20, 0);
    let shownTags = [];
    let extra = 0;
    const width = selectedGrid === 'song' ? widths.tags : widths2.tags;
    if (estWidth >= width) {
      const availWidth = width - 45;
      let total = 0;
      for (let tag of songs[rowIndex].tags) {
        total += tag.name.length * 6 + 20;
        if (total < availWidth) {
          shownTags.push(tag);
        } else {
          extra++;
        }
      }
    } else {
      shownTags = songs[rowIndex].tags;
    }
    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={e => onRowClick(e, path)}>
        {shownTags
          .sort(t => t.order)
          .map(t => (
            <GridTag tag={t} isLightTheme={isLightTheme} key={path + t.name} songId={songs[rowIndex].id} />
          ))}
        {extra > 0 ? (
          <Button style={{ minHeight: 20, maxHeight: 20, marginTop: 2 }} small minimal outlined intent={Intent.PRIMARY}>
            +{extra}
          </Button>
        ) : null}
      </div>
    );
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

  const onRowClick = (e: React.MouseEvent, path: string) => {
    onFileSelect(e, path);
    const cur = songs.filter(s => s.path === path)[0];
    let albumIndex = albumKeys.findIndex(v => v === cur.albumArtist + ' ' + cur.album);
    const hasArt = getAlbumSongs(albumIndex).filter(s => s.hasArt);
    const song = hasArt.length > 0 ? hasArt[0] : cur;
    updateSelectedAlbum(song.id, song.hasArt, albumIndex);

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

  const updateSelectedAlbum = async (songId: number, hasArt: boolean, albumIndex: number) => {
    if (hasArt) {
      await updateColors(songId, albumIndex);
    }
    setSelectedAlbum(albumKeys[albumIndex]);
  };

  const updateColors = async (songId: number, albumIndex: number) => {
    const colors = await loadColors(songId);
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
    songs
      .find(s => s.id === songId)
      ?.tags?.forEach(({ color }, i) => {
        const [r, g, b] = color.split(',').map(parseFloat);
        const first = { ...fg, a: 1 };
        const blended1 = normal(first, { r, g, b, a: 0.15 });
        const blended2 = normal(first, { r, g, b, a: 0.25 });
        setCssVar(`--tag-bg-${i + 1}`, formatRgb(blended1));
        setCssVar(`--tag-fg-${i + 1}`, formatRgb(blended2));
        console.log('here');
      });
  };

  const rowRenderer = (props: TableRowProps) => {
    props.className += ' row';
    return defaultTableRowRenderer(props);
  };

  const rowRenderer2 = (props: TableRowProps) => {
    props.className += ' card';
    props.style.left = 10;
    if (`${props.rowData[0].albumArtist} ${props.rowData[0].album}` === selectedAlbum) {
      props.className += ' album-selected-row';
    }
    if (groupedSongs[albumKeys[props.index]].filter(s => s.hasArt).length > 0) {
      props.className += ' has-art';
    }

    props.style.height -= 15;
    props.style.width -= 30;
    return defaultTableRowRenderer(props);
  };

  const onPlay = async () => {
    const fileToPlay = playingFile !== '' ? playingFile : selectedFiles[0];
    await startQueue(fileToPlay ?? '');
  };

  const multiSongRenderer = (props: TableCellProps, cellRenderer: (props: TableCellProps) => JSX.Element | null) => {
    let g = groupedSongs[albumKeys[props.rowIndex]];
    return <div className='rowParent'>{g.map(gg => cellRenderer({ ...props, rowIndex: gg.index }))}</div>;
  };

  const multiSongRenderer2 = (props: TableCellProps, draggingSong: string) => {
    let g = groupedSongs[albumKeys[props.rowIndex]];
    return <div className='rowParent'>{g.map(gg => genericCellRenderer2(gg.index, 'name', draggingSong))}</div>;
  };

  const otherGrid = (draggingSong: string) => (
    <div style={{ height }}>
      <Table
        className='main-grid'
        ref={ref => {
          if (ref) {
            otherRef.current = ref;
          }
        }}
        width={width - 5}
        height={height}
        headerHeight={25}
        rowCount={albumKeys.length}
        rowRenderer={rowRenderer2}
        overscanRowCount={0}
        estimatedRowSize={groupedSongs?.keys?.length > 0 ? (songs.length / groupedSongs.keys.length) * 25 : 250}
        rowHeight={index => Math.max(groupedSongs[albumKeys[index.index]].length * 25 + 40, 180)}
        rowGetter={({ index }) => groupedSongs[albumKeys[index]]}
      >
        <Column
          headerRenderer={headerRenderer}
          dataKey='album'
          label='Album'
          cellRenderer={({ rowIndex }) => {
            let gg = groupedSongs[albumKeys[rowIndex]];
            let hasArt = gg.filter(ggg => ggg.hasArt);
            let g = hasArt.length > 0 ? hasArt[0] : gg[0];
            return (
              <FlexCol
                center={false}
                style={{ paddingLeft: 10, height: Math.max(gg.length * 25, 140) }}
                onClick={() => updateSelectedAlbum(g.id, g.hasArt, rowIndex)}
              >
                {draggingSong === albumKeys[rowIndex] ? (
                  <>
                    <Text ellipsize>{g.albumArtist}</Text>
                    <div style={{ paddingBottom: 5 }}>
                      <Text ellipsize>{g.album}</Text>
                    </div>
                  </>
                ) : (
                  <Draggable draggableId={`album-${albumKeys[rowIndex]}`} index={rowIndex} key={albumKeys[rowIndex]}>
                    {(provided: DraggableProvided, snapshot: DraggableStateSnapshot) => {
                      return (
                        <div
                          ref={provided.innerRef}
                          {...provided.dragHandleProps}
                          {...provided.draggableProps}
                          style={{
                            ...provided.draggableProps.style,
                            ...provided.draggableProps.style,
                            transform: 'none',
                          }}
                        >
                          <Text ellipsize>{g.albumArtist}</Text>
                          <div style={{ paddingBottom: 5 }}>
                            <Text ellipsize>{g.album}</Text>
                          </div>
                        </div>
                      );
                    }}
                  </Draggable>
                )}

                {g.hasArt ? (
                  <img
                    loading='lazy'
                    alt={`${g.album} cover`}
                    src={`http://localhost:5000/albumArt?songId=${g.id}`}
                    width={75}
                    height={75}
                  />
                ) : null}
                <div style={{ paddingTop: 5, fontSize: 12 }}>
                  {formatMs(gg.map(g => g.time).reduce((prev, current) => prev + current))}
                </div>
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
          cellRenderer={props => multiSongRenderer(props, editCellRenderer)}
          width={widths2.edit}
          minWidth={widths2.edit}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='name'
          label='Title'
          cellRenderer={props => multiSongRenderer2(props, draggingSong)}
          width={widths2.name}
          minWidth={widths2.name}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='track'
          label='Track'
          cellRenderer={props => multiSongRenderer(props, trackRenderer)}
          width={widths2.track}
          minWidth={widths2.track}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='time'
          label='Time'
          cellRenderer={props => multiSongRenderer(props, timeRenderer)}
          width={widths2.time}
          minWidth={widths2.time}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey=''
          label='Tags'
          cellRenderer={props => multiSongRenderer(props, tagRenderer)}
          width={widths2.tags}
          minWidth={widths2.tags}
        />
      </Table>
    </div>
  );

  const mainGrid = (draggingSong: string) => (
    <div style={{ height }}>
      <Table
        className='main-grid'
        ref={ref => {
          if (ref) {
            mainRef.current = ref;
          }
        }}
        width={width - 5}
        height={height}
        headerHeight={25}
        rowHeight={25}
        rowCount={songs.length}
        rowRenderer={rowRenderer}
        rowGetter={({ index }) => songs[index]}
      >
        <Column
          headerRenderer={headerRenderer}
          dataKey=''
          cellRenderer={editCellRenderer}
          width={widths.edit}
          minWidth={widths.edit}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='name'
          label='Title'
          cellRenderer={({ rowIndex }) => genericCellRenderer2(rowIndex, 'name', draggingSong)}
          width={widths.name}
          minWidth={widths.name}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='albumArtist'
          label='Album Artist'
          cellRenderer={genericCellRenderer}
          width={widths.albumArtist}
          minWidth={widths.albumArtist}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='artist'
          label='Artist'
          cellRenderer={genericCellRenderer}
          width={widths.artist}
          minWidth={widths.artist}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='album'
          label='Album'
          cellRenderer={genericCellRenderer}
          width={widths.album}
          minWidth={widths.album}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='track'
          label='Track'
          cellRenderer={trackRenderer}
          width={widths.track}
          minWidth={widths.track}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='time'
          label='Time'
          cellRenderer={timeRenderer}
          width={widths.time}
          minWidth={widths.time}
        />
        <Column
          headerRenderer={headerRenderer}
          dataKey='path'
          label='Tags'
          cellRenderer={tagRenderer}
          width={widths.tags}
          minWidth={widths.tags}
        />
      </Table>
    </div>
  );

  return (
    <>
      <Droppable
        isDropDisabled
        droppableId='mainGrid'
        mode='virtual'
        renderClone={(provided: DraggableProvided, snapshot: DraggableStateSnapshot, rubric: DraggableRubric) => {
          return (
            <div
              {...provided.draggableProps}
              {...provided.dragHandleProps}
              ref={provided.innerRef}
              style={{ ...provided.draggableProps.style, background: 'rgba(var(--background-main), 0.7)' }}
            >
              <FlexRow style={{ paddingLeft: 5 }}>
                <FlexCol>
                  <Text ellipsize>
                    {selectedGrid === 'song'
                      ? songs[rubric.source.index].name
                      : groupedSongs[albumKeys[rubric.source.index]][0].artist}
                  </Text>
                  {selectedGrid === 'album' ? (
                    <Text ellipsize>{groupedSongs[albumKeys[rubric.source.index]][0].album}</Text>
                  ) : null}
                </FlexCol>

                {selectedFiles.length > 1 ? (
                  <FlexCol
                    style={{
                      borderRadius: '50%',
                      background: 'rgba(var(--intent-primary), 0.4)',
                      width: 20,
                      height: 20,
                      alignSelf: 'right',
                      position: 'absolute',
                      right: -10,
                      top: -10,
                    }}
                  >
                    {selectedFiles.length}
                  </FlexCol>
                ) : null}
              </FlexRow>
            </div>
          );
        }}
      >
        {(droppableProvided: DroppableProvided, snapshot: DroppableStateSnapshot) => {
          const node = ReactDOM.findDOMNode(selectedGrid === 'song' ? mainRef.current : otherRef.current);
          if (node instanceof HTMLElement) {
            droppableProvided.innerRef(node);
          }

          return (
            <div>
              {selectedGrid === 'song'
                ? mainGrid(snapshot.draggingFromThisWith ?? '')
                : otherGrid(snapshot.draggingFromThisWith ?? '')}
            </div>
          );
        }}
      </Droppable>
      <Controls
        onPlay={onPlay}
        onPrevious={onPrevious}
        playingSong={playingFile !== '' ? queuedSongs.filter(s => s.path === playingFile)[0] : null}
      />
    </>
  );
};
