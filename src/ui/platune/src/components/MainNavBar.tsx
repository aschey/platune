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
  Tag,
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
import { SideTag } from './SideTag';
import { shadeColorRgb } from '../themes/colorMixer';
import { useSelector } from 'react-redux';
import { useAppDispatch } from '../state/store';
import { FilterRequest } from '../models/filterRequest';
import { clearSearch, fetchSearchResults, selectSearchResults } from '../state/search';
import { GridType } from '../enums/gridType';
import { useThemeContext } from '../state/themeContext';
import { useFilters, useTagFilters } from '../hooks/useStore';

interface MainNavBarProps {
  sidePanelWidth: number;
  setSidePanelWidth: (width: number) => void;
  selectedGrid: GridType;
  setSelectedGrid: (selectedGrid: GridType) => void;
}
const MusicSuggest = Suggest.ofType<Search>();
const MusicOmnibar = Omnibar.ofType<Search>();

export const MainNavBar: React.FC<MainNavBarProps> = ({
  sidePanelWidth,
  setSidePanelWidth,
  selectedGrid,
  setSelectedGrid,
}) => {
  const [omnibarOpen, setOmnibarOpen] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [selectedSearch, setSelectedSearch] = useState<Search | null>();

  const dispatch = useAppDispatch();
  const searchResults = useSelector(selectSearchResults);
  const { setTheme, isLightTheme } = useThemeContext();
  const { setFilters } = useFilters();
  const { setFilterTag } = useTagFilters();

  const globalHotkeysEvents = new HotkeysEvents(HotkeyScope.GLOBAL);

  const updateSearch = useCallback(() => {
    if (!selectedSearch) {
      return;
    }
    let params: FilterRequest = {};
    switch (selectedSearch.entryType) {
      case 'song':
        params = {
          artistId: selectedSearch.correlationId,
          songName: selectedSearch.entryValue,
        };
        break;
      case 'album':
        params = {
          albumId: selectedSearch.correlationId,
        };
        break;
      case 'artist':
        params = {
          artistId: selectedSearch.correlationId,
        };
        break;
      case 'album_artist':
        params = {
          albumArtistId: selectedSearch.correlationId,
        };
        break;
    }
    setFilters(params);
  }, [selectedSearch, dispatch]);

  useEffect(() => {
    updateSearch();
  }, [selectedSearch, updateSearch]);

  const search = (searchString: string, includeTags: boolean) => {
    dispatch(fetchSearchResults({ searchString, limit: 10, includeTags }));
  };

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
        style={{
          //paddingBottom: props.index === searchResults.length - 1 ? 0 : 10,
          backgroundColor: active ? 'rgba(var(--intent-primary), 0.3)' : undefined,
        }}
        text={
          searchRes.entryType.toLowerCase() === 'tag' ? (
            <Tag
              minimal
              style={{
                border: `1px solid rgba(${searchRes.tagColor}, 0.25)`,
                backgroundColor: `rgba(${searchRes.tagColor}, 0.15)`,
                color: `rgba(${shadeColorRgb(searchRes.tagColor as string, isLightTheme ? -50 : 100)}, 1)`,
              }}
            >
              {searchRes.entryValue}
            </Tag>
          ) : (
            <>
              <div>{highlightText(searchRes.entryValue, props.query, active)}</div>

              <div
                style={{
                  fontSize: 12,
                  color: active ? 'rgba(255, 255, 255, 0.6)' : 'rgba(var(--text-secondary), 0.8)',
                }}
              >
                {searchRes.artist === null
                  ? searchRes.entryType.split('_').map(capitalize).join(' ')
                  : `${capitalize(searchRes.entryType)} by ${searchRes.artist}`}
              </div>
            </>
          )
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

  const resetSearch = useCallback(() => {
    toastMessage('Resetting...');
    setFilters({});
    setSelectedSearch(null);
  }, [setSelectedSearch, dispatch]);

  const toggleSideBar = useCallback(() => setSidePanelWidth(sidePanelWidth > 0 ? 0 : 200), [
    setSidePanelWidth,
    sidePanelWidth,
  ]);

  const setSelected = (val: Search) => {
    if (val.entryType === 'tag') {
      setFilterTag({ tagId: val.correlationId, append: false, toggle: false });
    } else {
      setSelectedSearch(val);
    }
  };

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
        onKeyDown: () => setSelectedGrid(GridType.Album),
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + l',
        label: 'Show song list',
        onKeyDown: () => setSelectedGrid(GridType.Song),
        preventDefault: true,
      }),
      new Hotkey({
        global: true,
        combo: 'shift + x',
        label: 'Clear search',
        onKeyDown: resetSearch,
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
  }, [resetSearch, omnibarOpen, toggleSideBar, dispatch]);

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
                  <MenuItem text={themeEntry('Dark', !isLightTheme)} onClick={() => setTheme('dark')} />
                  <MenuItem text={themeEntry('Light', isLightTheme)} onClick={() => setTheme('light')} />
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
          inputValueRenderer={val => (val.entryType === 'tag' ? '' : val.entryValue)}
          itemRenderer={searchItemRenderer}
          selectedItem={selectedSearch}
          initialContent='Type to search'
          onItemSelect={setSelected}
          items={searchResults}
          popoverProps={{ minimal: true }}
          itemsEqual={(first, second) => first.entryValue === second.entryValue && first.artist === second.artist}
          inputProps={{
            leftIcon: 'search',
            rightElement: <Button minimal icon='small-cross' onClick={resetSearch} />,
          }}
          onQueryChange={input => {
            search(input, false);
          }}
        />
        <MusicOmnibar
          resetOnSelect
          isOpen={omnibarOpen}
          itemRenderer={searchItemRenderer}
          items={searchResults}
          onItemSelect={val => {
            setOmnibarOpen(false);
            setSelected(val);
          }}
          onClose={() => setOmnibarOpen(false)}
          onQueryChange={input => {
            search(input, true);
          }}
        />
        <NavbarGroup align={Alignment.RIGHT} style={{ height: 40, paddingTop: 1 }}>
          <ButtonGroup minimal>
            <Button
              outlined
              style={{ width: 30, height: 30 }}
              intent={selectedGrid === GridType.Song ? Intent.PRIMARY : Intent.NONE}
              onClick={() => setSelectedGrid(GridType.Song)}
            >
              <FontAwesomeIcon icon={faThList} style={{ marginTop: 1.5 }} />
            </Button>
            <div style={{ width: 3 }}></div>
            <Button
              outlined
              intent={selectedGrid === GridType.Album ? Intent.PRIMARY : Intent.NONE}
              icon='list-detail-view'
              onClick={() => setSelectedGrid(GridType.Album)}
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
      <Settings isOpen={isOpen} setIsOpen={setIsOpen} />
    </>
  );
};
