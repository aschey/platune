import React, { useState, useEffect, useRef } from 'react';
import {
  Navbar,
  NavbarGroup,
  Alignment,
  NavbarHeading,
  NavbarDivider,
  Button,
  Classes,
  Popover,
  MenuItem,
  Menu,
  Position,
  Icon,
  ButtonGroup,
  Intent,
  HotkeysTarget,
  IHotkeysProps,
  Hotkeys,
  Hotkey,
} from '@blueprintjs/core';
import { Settings } from './Settings';
import luneDark from '../res/lune-text-dark.png';
import lune from '../res/lune-text.png';
import logo from '../res/drawing2.svg';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faThList, faTimes } from '@fortawesome/free-solid-svg-icons';
import { faSquare, faWindowMinimize, faWindowClose } from '@fortawesome/free-regular-svg-icons';
import { BrowserWindow, remote } from 'electron';
import { Suggest, Omnibar, IItemRendererProps } from '@blueprintjs/select';
import { Song } from '../models/song';
import { HotkeysEvents, HotkeyScope } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysEvents';
import { showHotkeysDialog } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysDialog';
import { getJson } from '../fetchUtil';
import { Search } from '../models/search';
import _, { capitalize } from 'lodash';
import { toastMessage } from '../appToaster';

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

  const getWindow = () => remote.BrowserWindow.getFocusedWindow();

  const hotkeys = (
    <Hotkeys>
      <Hotkey
        global
        combo='shift + o'
        label='Open omnibar'
        onKeyDown={() => setOmnibarOpen(!omnibarOpen)}
        preventDefault
      />
      <Hotkey
        global
        combo='shift + a'
        label='Show album grid'
        onKeyDown={() => setSelectedGrid('album')}
        preventDefault
      />
      <Hotkey
        global
        combo='shift + l'
        label='Show song list'
        onKeyDown={() => setSelectedGrid('song')}
        preventDefault
      />
    </Hotkeys>
  );
  const globalHotkeysEvents = new HotkeysEvents(HotkeyScope.GLOBAL);
  const debounced = _.debounce(async (input: string) => {
    let res = await getJson<Search[]>(
      `/search?limit=10&searchString=${input
        .split(/\s+/)
        .map(s => `"${s}"`)
        .join(' ')}*`
    );
    setSearchResults(res);
  });
  useEffect(() => {
    document.addEventListener('keydown', globalHotkeysEvents.handleKeyDown);
    document.addEventListener('keyup', globalHotkeysEvents.handleKeyUp);
    if (globalHotkeysEvents) {
      globalHotkeysEvents.setHotkeys(hotkeys.props);
    }

    return () => {
      document.removeEventListener('keydown', globalHotkeysEvents.handleKeyDown);
      document.removeEventListener('keyup', globalHotkeysEvents.handleKeyUp);

      globalHotkeysEvents.clear();
    };
  });

  const escapeRegExpChars = (text: string) => {
    return text.replace(/([.*+?^=!:${}()|\[\]\/\\])/g, '\\$1');
  };

  const searchTextColor = (active: boolean, alpha: number) =>
    active ? `rgba(255,255,255,${alpha})` : 'rgba(var(--text-main), ${alpha})';

  const highlightText = (text: string, query: string, active: boolean) => {
    let lastIndex = 0;
    const words = query
      .split(/\s+/)
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
    return <div style={{ color: searchTextColor(active, 0.9) }}>{tokens}</div>;
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
        getJson<Song[]>(`/songs?artistId=${val.correlationId}&songName=${val.entryValue}`).then(setSongs);
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

  return (
    <>
      <Navbar fixedToTop style={{ height: '40px', paddingRight: 5 }}>
        <NavbarGroup align={Alignment.LEFT} style={{ height: 40, paddingTop: 1 }}>
          <NavbarHeading style={{ marginRight: 0, marginTop: 4, paddingRight: 7 }}>
            <img src={logo} width={28} height={28} />
          </NavbarHeading>
          <NavbarDivider />
          <Popover
            autoFocus={false}
            content={
              <Menu>
                <MenuItem icon='cog' text='Settings' onClick={() => setIsOpen(true)} />
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
          <Button
            minimal
            icon={sidePanelWidth > 0 ? 'double-chevron-left' : 'double-chevron-right'}
            onClick={() => setSidePanelWidth(sidePanelWidth > 0 ? 0 : 200)}
          />
          <div style={{ width: 5 }} />
        </NavbarGroup>
        <MusicSuggest
          fill
          className='search'
          inputValueRenderer={val => val.entryValue}
          itemRenderer={searchItemRenderer}
          initialContent='Type to search'
          onItemSelect={(val, event) => {
            updateSearch(val);
          }}
          items={searchResults}
          popoverProps={{ minimal: true }}
          itemsEqual={(first, second) => first.entryValue === second.entryValue && first.artist === second.artist}
          inputProps={{
            leftIcon: 'search',
            rightElement: (
              <Button
                minimal
                icon='small-cross'
                onClick={() => {
                  toastMessage('Resetting...');
                  getJson<Song[]>('/songs').then(setSongs);
                }}
              />
            ),
          }}
          onQueryChange={async (input, event) => {
            await debounced(input);
          }}
        />
        <MusicOmnibar
          isOpen={omnibarOpen}
          itemRenderer={searchItemRenderer}
          items={searchResults}
          onItemSelect={(val, event) => {
            setOmnibarOpen(false);
            updateSearch(val);
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
            <Button intent={Intent.WARNING} className='hover-intent' onClick={() => getWindow()?.minimize()}>
              <FontAwesomeIcon icon={faWindowMinimize} />
            </Button>
            <Button
              intent={Intent.SUCCESS}
              className='hover-intent'
              style={{ transform: 'translate(0, 1px)' }}
              onClick={() => {
                const window = getWindow();
                if (window?.isMaximized()) {
                  window?.restore();
                } else {
                  window?.maximize();
                }
                window?.reload();
              }}
            >
              <FontAwesomeIcon icon={faSquare} />
            </Button>
            <Button intent={Intent.DANGER} className='hover-intent' onClick={() => getWindow()?.close()}>
              <FontAwesomeIcon icon={faTimes} style={{ transform: 'translate(0, 1px)' }} />
            </Button>
          </ButtonGroup>
        </NavbarGroup>
      </Navbar>
      <Settings updateTheme={updateTheme} isOpen={isOpen} setIsOpen={setIsOpen} />
    </>
  );
};
