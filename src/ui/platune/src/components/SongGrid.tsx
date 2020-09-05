import { Button, EditableText, Text, Tag, Intent, Colors } from '@blueprintjs/core';
import _, { Dictionary } from 'lodash';
import React, { useCallback, useEffect, useState } from 'react';
import Draggable from 'react-draggable';
import { Column, defaultTableRowRenderer, Table, TableHeaderProps, TableRowProps } from 'react-virtualized';
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

interface SongGridProps {
  selectedGrid: string;
  isLightTheme: boolean;
  width: number;
  height: number;
  songs: Song[];
  setSongs: (songs: Song[]) => void;
  queuedSongs: Song[];
  setQueuedSongs: (songs: Song[]) => void;
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
}) => {
  const [groupedSongs, setGroupedSongs] = useState<Dictionary<Song[]>>({});
  const [albumKeys, setAlbumKeys] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState('');
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

  const mainRef = React.createRef<Table>();
  const otherRef = React.createRef<Table>();
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
            onClick={() => {
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
                setSelectedFile(path);
                setEditingFile(path);
              }
            }}
          />
        </FlexCol>
      </div>
    );
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
    } else if (path === selectedFile) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }
    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={() => onRowClick(path)}>
        <Text ellipsize>{child}</Text>
      </div>
    );
  };

  const genericCellRenderer = (rowIndex: number, field: 'name' | 'albumArtist' | 'artist' | 'album' | 'time') => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    const value = songs[rowIndex][field].toString();
    return cellRenderer(rowIndex, path, value);
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
    return cellRenderer(rowIndex, path, value);
  };

  const timeRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex]['time'];
    let fmtValue = formatMs(value);
    return cellRenderer(rowIndex, path, fmtValue);
  };

  const pathRenderer = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex].path;

    let classes = 'bp3-table-cell grid-cell';
    //let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      //child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (path === selectedFile) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }

    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={() => onRowClick(path)}>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.PRIMARY}>
          Main
        </Tag>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.SUCCESS}>
          90s
        </Tag>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.DANGER}>
          Metal
        </Tag>
        <Text>+5</Text>
      </div>
    );
  };

  const pathRenderer2 = (rowIndex: number) => {
    if (rowIndex >= songs.length) {
      return null;
    }
    const path = songs[rowIndex].path;
    let value = songs[rowIndex].path;

    let classes = 'bp3-table-cell grid-cell';
    //let child: JSX.Element | string = value;
    if (path === editingFile) {
      classes += ' editing selected';
      //child = canEdit ? <EditableText defaultValue={value} className='editing' /> : value;
    } else if (path === playingFile) {
      classes += ' playing';
    } else if (path === selectedFile) {
      classes += ' selected';
    } else {
      classes += rowIndex % 2 === 0 ? ' striped-even' : ' striped-odd';
    }

    if (songs[rowIndex].hasArt) {
      return (
        <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={() => onRowClick(path)}>
          <div>
            <Tag
              style={{
                height: 20,
                marginTop: 2,
                marginRight: 5,
                background: 'rgba(var(--tag-bg-1))',
                color: 'rgba(var(--grid-selected-stripe-even))',
              }}
            >
              Main
            </Tag>
            <Tag
              style={{
                height: 20,
                marginTop: 2,
                marginRight: 5,
                background: 'rgba(var(--tag-bg-2))',
                color: 'rgba(var(--grid-selected-stripe-even))',
              }}
            >
              90s
            </Tag>
            <Tag
              style={{
                height: 20,
                marginTop: 2,
                marginRight: 5,
                background: 'rgba(var(--tag-bg-3))',
                color: 'rgba(var(--grid-selected-stripe-even))',
              }}
            >
              Metal
            </Tag>
            <Text>+5</Text>
          </div>
        </div>
      );
    }
    return (
      <div key={path} className={classes} onDoubleClick={() => onDoubleClick(path)} onClick={() => onRowClick(path)}>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.PRIMARY}>
          Main
        </Tag>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.SUCCESS}>
          90s
        </Tag>
        <Tag style={{ height: 20, marginTop: 2, marginRight: 5 }} intent={Intent.DANGER}>
          Metal
        </Tag>
        <Text>+5</Text>
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

  const onRowClick = (path: string) => {
    setSelectedFile(path);
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

  const updateSelectedAlbum = async (songIndex: number, hasArt: boolean, albumIndex: number) => {
    if (hasArt) {
      await updateColors(songIndex, albumIndex);
    }
    setSelectedAlbum(albumKeys[albumIndex]);
  };

  const updateColors = async (songIndex: number, albumIndex: number) => {
    const colors = await loadColors(songIndex);
    const bg = colors[0];
    const fg = colors[1];
    const secondary = colors[2];
    const blue = hexToRgb(Colors.BLUE3);
    const green = hexToRgb(Colors.GREEN3);
    const red = hexToRgb(Colors.RED3);

    setCssVar('--grid-selected-text-color', formatRgb(fg));
    setCssVar('--grid-selected-shadow-1', formatRgb(bg));
    setCssVar('--grid-selected-shadow-2', formatRgb(bg));
    setCssVar('--grid-selected-stripe-even', formatRgb(bg));
    setCssVar('--grid-selected-background', formatRgb(secondary));
    setCssVar('--grid-selected-playing-row-background', formatRgb(colors[3]));
    setCssVar('--grid-selected-editing-row-color', formatRgb(colors[4]));

    const blended1 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: blue[0], g: blue[1], b: blue[2], a: 0.2 });
    const blended2 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: green[0], g: green[1], b: green[2], a: 0.2 });
    const blended3 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: red[0], g: red[1], b: red[2], a: 0.2 });

    setCssVar('--tag-bg-1', formatRgb(blended1));
    setCssVar('--tag-bg-2', formatRgb(blended2));
    setCssVar('--tag-bg-3', formatRgb(blended3));
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
    const fileToPlay = playingFile !== '' ? playingFile : selectedFile;
    await startQueue(fileToPlay ?? '');
  };

  const multiSongRenderer = (rowIndex: number, cellRenderer: (index: number) => JSX.Element | null) => {
    let g = groupedSongs[albumKeys[rowIndex]];
    return <div className='rowParent'>{g.map(gg => cellRenderer(gg.index))}</div>;
  };

  const multiSongRendererAsync = (rowIndex: number, cellRenderer: (index: number) => Promise<JSX.Element | null>) => {
    let g = groupedSongs[albumKeys[rowIndex]];
    return <div className='rowParent'>{g.map(async gg => await cellRenderer(gg.index))}</div>;
  };

  const otherGrid = (
    <div style={{ height }}>
      <Table
        ref={otherRef}
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
                <Text ellipsize>{g.albumArtist}</Text>
                <div style={{ paddingBottom: 5 }}>
                  <Text ellipsize>{g.album}</Text>
                </div>

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
          dataKey=''
          label='Tags'
          cellRenderer={({ rowIndex }) => multiSongRenderer(rowIndex, pathRenderer2)}
          width={widths2.tags}
          minWidth={widths2.tags}
        />
      </Table>
    </div>
  );

  const mainGrid = (
    <div style={{ height }}>
      <Table
        ref={mainRef}
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
          label='Tags'
          cellRenderer={({ rowIndex }) => pathRenderer(rowIndex)}
          width={widths.tags}
          minWidth={widths.tags}
        />
      </Table>
    </div>
  );

  return (
    <>
      <div>{selectedGrid === 'song' ? mainGrid : otherGrid}</div>
      <Controls
        onPlay={onPlay}
        onPrevious={onPrevious}
        playingSong={playingFile !== '' ? queuedSongs.filter(s => s.path === playingFile)[0] : null}
      />
    </>
  );
};
