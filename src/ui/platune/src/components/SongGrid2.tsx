import React, { useCallback, useEffect, useState } from 'react';
import BaseTable, { Column, AutoResizer } from 'react-base-table';
import { Song } from '../models/song';
import { range, sleep, formatRgb, setCssVar } from '../util';
import { getJson } from '../fetchUtil';
import { Dictionary } from 'lodash';
import { useObservable } from 'rxjs-hooks';
import { audioQueue } from '../audio';
import { Text, EditableText, Colors, Button } from '@blueprintjs/core';
import { hexToRgb } from '../themes/colorMixer';
import { Rgb } from '../models/rgb';
import { normal } from 'color-blend';
import { toastSuccess } from '../appToaster';
import { FlexCol } from './FlexCol';
import { Draggable, DraggableProvided, Droppable } from 'react-beautiful-dnd';

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

export const SongGrid2: React.FC<SongGridProps> = ({
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
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedAlbum, setSelectedAlbum] = useState('');
  const [editingFile, setEditingFile] = useState('');

  const mainRef = React.createRef<BaseTable>();
  const otherRef = React.createRef<BaseTable>();
  const playingFile = useObservable(() => audioQueue.playingSource);

  const loadSongs = useCallback(async () => {
    const numTries = 10;
    for (let i of range(numTries)) {
      try {
        const songs = await getJson<Song[]>('/songs');

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

  useEffect(() => {
    loadSongs().then(setSongs);
  }, [loadSongs, setSongs]);

  const getAlbumSongs = (albumIndex: number) => groupedSongs[albumKeys[albumIndex]];

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

  const cellRenderer = ({ path }: Song, value: string, rowIndex: number, canEdit: boolean = true) => {
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
      <Draggable draggableId={rowIndex.toString()} index={rowIndex} key={rowIndex}>
        {(provided: DraggableProvided) => (
          <div
            ref={provided.innerRef}
            {...provided.draggableProps}
            {...provided.dragHandleProps}
            key={path}
            className={classes}
            onDoubleClick={() => onDoubleClick(path)}
            onClick={e => onRowClick(e, path)}
          >
            <Text ellipsize>{child}</Text>
          </div>
        )}
      </Draggable>
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
          setSelectedFiles(selectedFiles.concat(paths.slice(i, index + 1)));
          break;
        }
      }
    } else {
      setSelectedFiles([path]);
    }
  };

  const loadColors = async (songId: number) => {
    const colors = await getJson<Rgb[]>(`/albumArtColors?songId=${songId}&isLight=${isLightTheme}`);
    return colors;
  };

  const updateColors = async (songIndex: number) => {
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

    const blended1 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: blue[0], g: blue[1], b: blue[2], a: 0.15 });
    const blended2 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: green[0], g: green[1], b: green[2], a: 0.15 });
    const blended3 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: red[0], g: red[1], b: red[2], a: 0.15 });

    const blended4 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: blue[0], g: blue[1], b: blue[2], a: 0.25 });
    const blended5 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: green[0], g: green[1], b: green[2], a: 0.25 });
    const blended6 = normal({ r: fg.r, g: fg.g, b: fg.g, a: 1 }, { r: red[0], g: red[1], b: red[2], a: 0.25 });

    setCssVar('--tag-bg-1', formatRgb(blended1));
    setCssVar('--tag-bg-2', formatRgb(blended2));
    setCssVar('--tag-bg-3', formatRgb(blended3));

    setCssVar('--tag-fg-1', formatRgb(blended4));
    setCssVar('--tag-fg-2', formatRgb(blended5));
    setCssVar('--tag-fg-3', formatRgb(blended6));
  };

  const updateSelectedAlbum = async (songIndex: number, hasArt: boolean, albumIndex: number) => {
    if (hasArt) {
      await updateColors(songIndex);
    }
    setSelectedAlbum(albumKeys[albumIndex]);
  };

  const editCellRenderer = (path: string, rowIndex: number) => {
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

  const genericCellRenderer = ({ rowData, cellData, rowIndex }: { rowData: any; cellData: any; rowIndex: number }) =>
    cellRenderer(rowData as Song, cellData, rowIndex);

  return (
    <Droppable droppableId='droppable' mode='virtual'>
      {(droppableProvided, droppableSnapshot) => (
        <div ref={droppableProvided.innerRef}>
          <AutoResizer>
            {({ width, height }) => (
              <BaseTable data={songs} width={width} height={height}>
                <Column
                  key='edit'
                  title=''
                  dataKey='path'
                  width={50}
                  cellRenderer={({ rowIndex, cellData }) => editCellRenderer(cellData, rowIndex)}
                />
                <Column
                  key='name'
                  title='Name'
                  dataKey='name'
                  width={0}
                  flexGrow={1}
                  resizable
                  sortable
                  cellRenderer={genericCellRenderer}
                />
                <Column
                  key='albumArtist'
                  title='Album Artist'
                  dataKey='albumArtist'
                  width={0}
                  flexGrow={1}
                  resizable
                  sortable
                  cellRenderer={genericCellRenderer}
                />
                <Column
                  key='artist'
                  title='Artist'
                  dataKey='artist'
                  width={0}
                  flexGrow={1}
                  resizable
                  sortable
                  cellRenderer={genericCellRenderer}
                />
                <Column
                  key='album'
                  title='Album'
                  dataKey='album'
                  width={0}
                  flexGrow={1}
                  resizable
                  sortable
                  cellRenderer={genericCellRenderer}
                />
              </BaseTable>
            )}
          </AutoResizer>
          {droppableProvided.placeholder}
        </div>
      )}
    </Droppable>
  );
};
