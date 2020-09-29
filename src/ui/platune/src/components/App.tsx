import { wrapGrid } from 'animate-css-grid';
import React, { useEffect, useState, useCallback, createContext, useMemo, useContext } from 'react';
import { Song } from '../models/song';
import { isLight } from '../themes/colorMixer';
import { darkTheme } from '../themes/dark';
import { lightTheme } from '../themes/light';
import { applyTheme } from '../themes/themes';
import { MainNavBar } from './MainNavBar';
import { QueueGrid } from './QueueGrid';
import { SongGrid } from './SongGrid';
import _, { initial } from 'lodash';
import { setCssVar } from '../util';
import { DragDropContext, DragStart, DropResult, ResponderProvided } from 'react-beautiful-dnd';
import { getJson, putJson } from '../fetchUtil';
import { toastSuccess } from '../appToaster';
import { SongTag } from '../models/songTag';
import { Search } from '../models/search';
import { batch, useDispatch, useSelector } from 'react-redux';
import { fetchSongs, selectSongs } from '../state/songs';
import { useAppDispatch } from '../state/store';
import { addSongsToTag, fetchTags } from '../state/songs';
import { GridType } from '../enums/gridType';
import { ThemeContextProvider } from '../state/themeContext';

const themeName = 'dark';
applyTheme(themeName);

const App: React.FC<{}> = () => {
  const [sidePanelWidth, setSidePanelWidth] = useState(0);
  const [gridCols, setGridCols] = useState(`0px ${window.innerWidth}px`);
  const [gridClasses, setGridClasses] = useState('grid');
  const [queuedSongs, setQueuedSongs] = useState<Song[]>([]);
  const [gridMargin, setGridMargin] = useState(0);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedGrid, setSelectedGrid] = useState(GridType.Song);

  const dispatch = useAppDispatch();
  const songs = useSelector(selectSongs);

  const getWidth = useCallback(() => window.innerWidth - gridMargin, [gridMargin]);
  const getHeight = () => window.innerHeight - 110;

  const [width, setWidth] = useState(getWidth());
  const [height, setHeight] = useState(getHeight());

  const gridRef = React.createRef<HTMLDivElement>();

  const debounced = _.debounce(async () => {
    setWidth(getWidth());
    setHeight(getHeight());
  }, 5);

  useEffect(() => {
    let w = window as any;
    //w['__react-beautiful-dnd-disable-dev-warnings'] = true;

    window.addEventListener('resize', debounced);
    return () => window.removeEventListener('resize', debounced);
  });

  useEffect(() => {
    setGridCols(`${sidePanelWidth}px ${window.innerWidth - sidePanelWidth}px`);
  }, [sidePanelWidth, width]);

  useEffect(() => {
    // this can trigger excessively because of the gridRef dependency
    // Make sure the side panel actually changed before running
    if (gridRef.current && sidePanelWidth !== gridMargin) {
      const { unwrapGrid } = wrapGrid(gridRef.current, {
        duration: 150,
        easing: 'linear',
        onStart: () => {
          setCssVar(
            '--table-transition',
            'color 1s, background 1s, border 1s, box-shadow 1s, width 150ms, min-width 150ms, max-width 150ms'
          );
          if (gridMargin > 0) {
            setWidth(window.innerWidth);
            setGridMargin(0);
          } else {
            setWidth(window.innerWidth - 200);
          }
        },
        onEnd: () => {
          if (gridMargin === 0) {
            setGridMargin(200);
          }
          setCssVar('--table-transition', 'color 1s, background 1s, border 1s, box-shadow 1s');
        },
      });
      // Remove animations after resizing because they don't play nicely with the virtualized grid
      setTimeout(unwrapGrid, 1);
    }

    if (sidePanelWidth > 0) {
      setGridClasses('expanded');
    } else {
      setGridClasses('collapsed');
    }
  }, [sidePanelWidth, gridMargin, getWidth, gridRef]);

  const onDragEnd = async ({ source, destination, draggableId }: DropResult) => {
    if (source.droppableId === 'mainGrid' && destination?.droppableId?.startsWith('tag-')) {
      const tagId = parseInt(destination.droppableId.split('-')[1]);
      let songIds: number[];
      if (draggableId.startsWith('album-')) {
        let albumKey = draggableId.replace('album-', '');
        songIds = songs.filter(s => `${s.albumArtist} ${s.album}` === albumKey).map(s => s.id);
      } else if (selectedFiles.includes(draggableId)) {
        songIds = songs.filter(s => selectedFiles.includes(s.path)).map(s => s.id);
      } else {
        songIds = songs.filter(s => draggableId === s.path).map(s => s.id);
      }
      dispatch(addSongsToTag({ tagId, songIds }));

      toastSuccess();
    }
  };

  const onBeforeDragStart = (initial: DragStart) => {
    if (initial.source.droppableId === 'mainGrid') {
      if (selectedFiles.length && selectedFiles.indexOf(initial.draggableId) === -1) {
        setSelectedFiles([]);
      }
    }
  };

  return (
    <ThemeContextProvider>
      <DragDropContext onDragEnd={onDragEnd} onBeforeDragStart={onBeforeDragStart}>
        <MainNavBar
          sidePanelWidth={sidePanelWidth}
          setSidePanelWidth={setSidePanelWidth}
          selectedGrid={selectedGrid}
          setSelectedGrid={setSelectedGrid}
        />
        <div
          ref={gridRef}
          className={gridClasses}
          style={{
            paddingTop: 40,
            display: 'grid',
            gridTemplateRows: `${height}px 70px`,
            gridTemplateColumns: gridCols,
          }}
        >
          <div>
            <div style={{ display: sidePanelWidth > 0 ? 'block' : 'none' }}>
              <QueueGrid queuedSongs={queuedSongs} />
            </div>
          </div>
          <SongGrid
            width={width}
            height={height}
            queuedSongs={queuedSongs}
            setQueuedSongs={setQueuedSongs}
            selectedFiles={selectedFiles}
            setSelectedFiles={setSelectedFiles}
            selectedGrid={selectedGrid}
          />
        </div>
      </DragDropContext>
    </ThemeContextProvider>
  );
};

export default App;
