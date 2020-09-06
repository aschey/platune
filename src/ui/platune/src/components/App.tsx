import { wrapGrid } from 'animate-css-grid';
import React, { useEffect, useState, useCallback } from 'react';
import { Song } from '../models/song';
import { isLight } from '../themes/colorMixer';
import { darkTheme } from '../themes/dark';
import { lightTheme } from '../themes/light';
import { applyTheme } from '../themes/themes';
import { MainNavBar } from './MainNavBar';
import { QueueGrid } from './QueueGrid';
import { SongGrid } from './SongGrid';
import _ from 'lodash';
import { setCssVar } from '../util';

const themeName = 'dark';
export const theme = darkTheme;
applyTheme(themeName);

const App: React.FC<{}> = () => {
  const [selectedGrid, setSelectedGrid] = useState('song');
  const [themeDetails, setThemeDetails] = useState(isLight(theme.backgroundMain));
  const [sidePanelWidth, setSidePanelWidth] = useState(0);
  const [gridCols, setGridCols] = useState(`0px ${window.innerWidth}px`);
  const [gridClasses, setGridClasses] = useState('grid');
  const [songs, setSongs] = useState<Song[]>([]);
  const [queuedSongs, setQueuedSongs] = useState<Song[]>([]);
  const [gridMargin, setGridMargin] = useState(0);

  const getWidth = useCallback(() => window.innerWidth - gridMargin, [gridMargin]);
  const getHeight = () => window.innerHeight - 110;

  const [width, setWidth] = useState(getWidth());
  const [height, setHeight] = useState(getHeight());

  const gridRef = React.createRef<HTMLDivElement>();

  const updateTheme = (newThemeName: string) => {
    applyTheme(newThemeName);
    const newTheme = newThemeName === 'light' ? lightTheme : darkTheme;
    setThemeDetails(isLight(newTheme.backgroundMain));
  };

  const debounced = _.debounce(async () => {
    setWidth(getWidth());
    setHeight(getHeight());
  }, 5);

  useEffect(() => {
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

  return (
    <>
      <MainNavBar
        sidePanelWidth={sidePanelWidth}
        setSidePanelWidth={setSidePanelWidth}
        selectedGrid={selectedGrid}
        setSelectedGrid={setSelectedGrid}
        updateTheme={updateTheme}
        isLight={themeDetails}
        songs={songs}
        setSongs={setSongs}
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
          selectedGrid={selectedGrid}
          isLightTheme={themeDetails}
          width={width}
          height={height}
          songs={songs}
          setSongs={setSongs}
          queuedSongs={queuedSongs}
          setQueuedSongs={setQueuedSongs}
        />
      </div>
    </>
  );
};

export default App;
