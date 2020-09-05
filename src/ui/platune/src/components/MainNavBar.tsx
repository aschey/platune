import {
  Alignment,
  Button,
  ButtonGroup,
  Hotkey,
  Icon,
  Intent,
  Menu,
  MenuItem,
  Navbar,
  NavbarDivider,
  NavbarGroup,
  NavbarHeading,
  Popover,
} from '@blueprintjs/core';
import { HotkeyScope, HotkeysEvents } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysEvents';
import { IItemRendererProps, Omnibar, Suggest } from '@blueprintjs/select';
import { faSquare, faWindowMinimize } from '@fortawesome/free-regular-svg-icons';
import { faThList, faTimes } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { ipcRenderer } from 'electron';
import _, { capitalize } from 'lodash';
import React, { useState, useEffect, useCallback } from 'react';
import { toastMessage } from '../appToaster';
import { getJson, putJson } from '../fetchUtil';
import { Search } from '../models/search';
import { Song } from '../models/song';
import { Settings } from './Settings';
import { globalHotkeys } from '../globalHotkeys';

interface MainNavBarProps {
  setSelectedGrid: (grid: string) => void;
  selectedGrid: string;
  updateTheme: (newThemeName: string) => void;
  isLight: boolean;
  sidePanelWidth: number;
  setSidePanelWidth: (width: number) => void;
  songs: Song[];
  setSongs: (songs: Song[]) => void;
}
const MusicSuggest = Suggest.ofType<Search>();
const MusicOmnibar = Omnibar.ofType<Search>();

export const MainNavBar: React.FC<MainNavBarProps> = ({
  selectedGrid,
  setSelectedGrid,
  updateTheme,
  isLight,
  sidePanelWidth,
  setSidePanelWidth,
  songs,
  setSongs,
}) => {
  const [omnibarOpen, setOmnibarOpen] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [searchResults, setSearchResults] = useState<Search[]>([]);
  const [selectedSearch, setSelectedSearch] = useState<Search | null>(null);

  const globalHotkeysEvents = new HotkeysEvents(HotkeyScope.GLOBAL);
  const debounced = _.debounce(async (input: string) => {
    let res = await getJson<Search[]>(
      `/search?limit=10&searchString=${encodeURIComponent(
        input
          .split(/\s+/)
          .map(s => `"${s}"`)
          .join(' ')
      )}*`
    );
    setSearchResults(res);
  });

  const escapeRegExpChars = (text: string) => {
    return text.replace(/([.*+?^=!:${}()|[\]/\\])/g, '\\$1');
  };

  const searchTextColor = (active: boolean, alpha: number) =>
    active ? `rgba(255,255,255,${alpha})` : `rgba(var(--text-main), ${alpha})`;

  const highlightText = (text: string, query: string, active: boolean) => {
    let lastIndex = 0;
    const words = query
      .split(/[^a-zA-Z0-9']+/)
      .filter(word => word.length > 0)
      .map(escapeRegExpChars);
    if (words.length === 0) {
      return [text];
    }
    const regexp = new RegExp(words.join('|'), 'gi');
    const tokens: React.ReactNode[] = [];
    while (true) {
      const match = regexp.exec(text);
      if (!match) {
        break;
      }
      const length = match[0].length;
      const before = text.slice(lastIndex, regexp.lastIndex - length);
      if (before.length > 0) {
        tokens.push(before);
      }
      lastIndex = regexp.lastIndex;
      tokens.push(
        <strong style={{ color: searchTextColor(active, 1) }} key={lastIndex}>
          {match[0]}
        </strong>
      );
    }
    const rest = text.slice(lastIndex);
    if (rest.length > 0) {
      tokens.push(rest);
    }
    return <div style={{ color: searchTextColor(active, 0.85) }}>{tokens}</div>;
  };

  const searchItemRenderer = (searchRes: Search, props: IItemRendererProps) => {
    const active = props.modifiers.active;
    return (
      <MenuItem
        key={props.index}
        active={active}
        onClick={props.handleClick}
        style={{ paddingBottom: props.index === searchResults.length - 1 ? 0 : 10 }}
        text={
          <>
            <div>{highlightText(searchRes.entryValue, props.query, active)}</div>
            <div
              style={{ fontSize: 12, color: active ? 'rgba(255, 255, 255, 0.6)' : 'rgba(var(--text-secondary), 0.8)' }}
            >
              {searchRes.artist === null
                ? searchRes.entryType.split('_').map(capitalize).join(' ')
                : `${capitalize(searchRes.entryType)} by ${searchRes.artist}`}
            </div>
          </>
        }
      />
    );
  };

  const themeEntry = (text: string, isSelected: boolean) => {
    return (
      <div>
        {text}
        {isSelected ? <Icon style={{ paddingLeft: 5 }} icon='tick' /> : null}
      </div>
    );
  };

  const updateSearch = (val: Search) => {
    switch (val.entryType) {
      case 'song':
        getJson<Song[]>(`/songs?artistId=${val.correlationId}&songName=${encodeURIComponent(val.entryValue)}`).then(
          setSongs
        );
        break;
      case 'album':
        getJson<Song[]>(`/songs?albumId=${val.correlationId}`).then(setSongs);
        break;
      case 'artist':
        getJson<Song[]>(`/songs?artistId=${val.correlationId}`).then(setSongs);
        break;
      case 'album_artist':
        getJson<Song[]>(`/songs?albumArtistId=${val.correlationId}`).then(setSongs);
        break;
    }
  };

  const clearSearch = useCallback(async () => {
    toastMessage('Resetting...');
    const allSongs = await getJson<Song[]>('/songs');
    setSongs(allSongs);
    setSearchResults([]);
    setSelectedSearch(null);
  }, [setSongs]);

  const toggleSideBar = useCallback(() => setSidePanelWidth(sidePanelWidth > 0 ? 0 : 200), [
    setSidePanelWidth,
    sidePanelWidth,
  ]);

  useEffect(() => {
    globalHotkeys.register([
      new Hotkey({
        global: true,
        combo: 'shift + o',
        label: 'Open omnibar',
        onKeyDown: () => setOmnibarOpen(!omnibarOpen),
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + a',
        label: 'Show album grid',
        onKeyDown: () => setSelectedGrid('album'),
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + l',
        label: 'Show song list',
        onKeyDown: () => setSelectedGrid('song'),
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + x',
        label: 'Clear search',
        onKeyDown: clearSearch,
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + s',
        label: 'Toggle sidebar',
        onKeyDown: toggleSideBar,
        preventDefault: true,
      }),
    ]);
  }, [clearSearch, omnibarOpen, setSelectedGrid, toggleSideBar]);

  return (
    <>
      <Navbar fixedToTop style={{ height: '40px', paddingRight: 5 }}>
        <NavbarGroup align={Alignment.LEFT} style={{ height: 40, paddingTop: 1 }}>
          <NavbarHeading style={{ marginRight: 0, marginTop: 4, paddingRight: 7 }}>
            <img src={`${process.env.PUBLIC_URL}/res/logo.svg`} alt='platune logo' width={28} height={28} />
          </NavbarHeading>
          <NavbarDivider />
          <Button
            minimal
            icon={sidePanelWidth > 0 ? 'double-chevron-left' : 'double-chevron-right'}
            onClick={toggleSideBar}
          />

          <div style={{ width: 5 }} />
          <Popover
            autoFocus={false}
            content={
              <Menu>
                <MenuItem icon='cog' text='Settings' onClick={() => setIsOpen(true)} />
                <MenuItem
                  icon='refresh'
                  text='Sync'
                  onClick={async () => {
                    await putJson<{}>('/sync', {});
                    toastMessage('Sync started');
                  }}
                />
                <MenuItem
                  icon='help'
                  text='Hotkeys'
                  onClick={() =>
                    // Hack to trigger the hotkey menu because sending a keyboard event doesn't set "which" properly
                    globalHotkeysEvents.handleKeyDown({ which: 191, shiftKey: true } as any)
                  }
                />
                <MenuItem icon='updated' text='Backup Now' />
                <MenuItem icon='exchange' text='Switch Theme'>
                  <MenuItem text={themeEntry('Dark', !isLight)} onClick={() => updateTheme('dark')} />
                  <MenuItem text={themeEntry('Light', isLight)} onClick={() => updateTheme('light')} />
                </MenuItem>
              </Menu>
            }
          >
            <Button minimal icon='menu' />
          </Popover>

          <div style={{ width: 5 }} />
        </NavbarGroup>
        <MusicSuggest
          fill
          resetOnSelect
          className='search'
          inputValueRenderer={val => val.entryValue}
          itemRenderer={searchItemRenderer}
          selectedItem={selectedSearch}
          initialContent='Type to search'
          onItemSelect={(val, event) => {
            updateSearch(val);
            setSelectedSearch(val);
          }}
          items={searchResults}
          popoverProps={{ minimal: true }}
          itemsEqual={(first, second) => first.entryValue === second.entryValue && first.artist === second.artist}
          inputProps={{
            leftIcon: 'search',
            rightElement: <Button minimal icon='small-cross' onClick={clearSearch} />,
          }}
          onQueryChange={async (input, event) => {
            await debounced(input);
          }}
        />
        <MusicOmnibar
          resetOnSelect
          isOpen={omnibarOpen}
          itemRenderer={searchItemRenderer}
          items={searchResults}
          onItemSelect={(val, event) => {
            setOmnibarOpen(false);
            updateSearch(val);
            setSelectedSearch(val);
          }}
          onClose={() => setOmnibarOpen(false)}
          onQueryChange={async (input, event) => {
            await debounced(input);
          }}
        />
        <NavbarGroup align={Alignment.RIGHT} style={{ height: 40, paddingTop: 1 }}>
          <ButtonGroup minimal>
            <Button
              outlined
              style={{ width: 30, height: 30 }}
              intent={selectedGrid === 'song' ? Intent.PRIMARY : Intent.NONE}
              onClick={() => setSelectedGrid('song')}
            >
              <FontAwesomeIcon icon={faThList} style={{ marginTop: 1.5 }} />
            </Button>
            <div style={{ width: 3 }}></div>
            <Button
              outlined
              intent={selectedGrid === 'album' ? Intent.PRIMARY : Intent.NONE}
              icon='list-detail-view'
              onClick={() => setSelectedGrid('album')}
            />
          </ButtonGroup>
          <div style={{ width: 20 }} />
          <ButtonGroup minimal>
            <Button intent={Intent.WARNING} className='hover-intent' onClick={() => ipcRenderer.invoke('minimize')}>
              <FontAwesomeIcon icon={faWindowMinimize} />
            </Button>
            <Button
              intent={Intent.SUCCESS}
              className='hover-intent'
              style={{ transform: 'translate(0, 1px)' }}
              onClick={() => ipcRenderer.invoke('restoreMax')}
            >
              <FontAwesomeIcon icon={faSquare} />
            </Button>
            <Button intent={Intent.DANGER} className='hover-intent' onClick={() => ipcRenderer.invoke('close')}>
              <FontAwesomeIcon icon={faTimes} style={{ transform: 'translate(0, 1px)' }} />
            </Button>
          </ButtonGroup>
        </NavbarGroup>
      </Navbar>
      <Settings updateTheme={updateTheme} isOpen={isOpen} setIsOpen={setIsOpen} />
    </>
  );
};
