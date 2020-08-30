import { wrapGrid } from 'animate-css-grid';
import React, { useEffect, useState } from 'react';
import { Song } from '../models/song';
import { isLight } from '../themes/colorMixer';
import { darkTheme } from '../themes/dark';
import { lightTheme } from '../themes/light';
import { applyTheme } from '../themes/themes';
import { MainNavBar } from './MainNavBar';
import { QueueGrid } from './QueueGrid';
import { SongGrid } from './SongGrid';

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

  const gridRef = React.createRef<HTMLDivElement>();

  const updateTheme = (newThemeName: string) => {
    applyTheme(newThemeName);
    const newTheme = newThemeName === 'light' ? lightTheme : darkTheme;
    setThemeDetails(isLight(newTheme.backgroundMain));
  };

  useEffect(() => {
    if (gridRef.current) {
      const { unwrapGrid } = wrapGrid(gridRef.current, {
        duration: 150,
        onStart: () => {
          if (gridMargin > 0) {
            setGridMargin(0);
          }
        },
        onEnd: () => {
          if (gridMargin === 0) {
            setGridMargin(200);
          }
        },
      });
      // Remove animations after resizing because they don't play nicely with the virtualized grid
      setTimeout(unwrapGrid, 1);
    }
    setGridCols(`${sidePanelWidth}px ${window.innerWidth - sidePanelWidth}px`);
    if (sidePanelWidth > 0) {
      setGridClasses('expanded');
    } else {
      setGridClasses('collapsed');
    }
  }, [sidePanelWidth, gridMargin, gridRef]);

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
          gridTemplateRows: `${window.innerHeight - 110}px 70px`,
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
          width={window.innerWidth - gridMargin}
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
