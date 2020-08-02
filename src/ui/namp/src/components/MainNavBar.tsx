import React, { useState, useEffect, useRef } from 'react';
import { Navbar, NavbarGroup, Alignment, NavbarHeading, NavbarDivider, Button, Classes, Popover, MenuItem, Menu, Position, Icon, ButtonGroup, Intent, HotkeysTarget, IHotkeysProps, Hotkeys, Hotkey } from '@blueprintjs/core';
import { Settings } from './Settings';
import luneDark from '../res/lune-text-dark.png';
import lune from '../res/lune-text.png';
import logo from '../res/drawing2.svg';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faThList, faTimes } from '@fortawesome/free-solid-svg-icons'
import { faSquare, faWindowMinimize, faWindowClose } from '@fortawesome/free-regular-svg-icons'
import { BrowserWindow, remote } from 'electron';
import { Suggest, Omnibar, IItemRendererProps } from '@blueprintjs/select';
import { Song } from '../models/song';
import { HotkeysEvents, HotkeyScope } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysEvents';
import { showHotkeysDialog } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysDialog';
import { getJson } from '../fetchUtil';
import { Search } from '../models/search';
import _, { capitalize } from 'lodash';

interface MainNavBarProps {
    setSelectedGrid: (grid: string) => void;
    selectedGrid: string;
    updateTheme: (newThemeName: string) => void,
    isLight: boolean
}
const MusicSuggest = Suggest.ofType<Search>();
const MusicOmnibar = Omnibar.ofType<Search>();

export const MainNavBar: React.FC<MainNavBarProps> = ({ selectedGrid, setSelectedGrid, updateTheme, isLight }) => {
    const [omnibarOpen, setOmnibarOpen] = useState(false);
    const [isOpen, setIsOpen] = useState(false);
    const [searchResults, setSearchResults] = useState<Search[]>([]);
    const getWindow = () => remote.BrowserWindow.getFocusedWindow();
    const hotkeys = <Hotkeys>
        <Hotkey
            global={true}
            combo="shift + o"
            label="Open omnibar"
            onKeyDown={() => setOmnibarOpen(!omnibarOpen)}
            preventDefault={true}
        />
        <Hotkey
            global={true}
            combo="shift + a"
            label="Show album grid"
            onKeyDown={() => setSelectedGrid('album')}
            preventDefault={true}
        />
        <Hotkey
            global={true}
            combo="shift + l"
            label="Show song list"
            onKeyDown={() => setSelectedGrid('song')}
            preventDefault={true}
        />
    </Hotkeys>;
    const globalHotkeysEvents = new HotkeysEvents(HotkeyScope.GLOBAL);
    const debounced = _.debounce(async (input: string) => {
        let res = await getJson<Search[]>(`/search?limit=10&searchString=${input}*`);
        setSearchResults(res);
    });
    useEffect(() => {
        console.log('here');
        document.addEventListener("keydown", globalHotkeysEvents.handleKeyDown);
        document.addEventListener("keyup", globalHotkeysEvents.handleKeyUp);
        if (globalHotkeysEvents) {
            globalHotkeysEvents.setHotkeys(hotkeys.props);
        }
        if (searchResults === []) {
            setSearchResults([{entryType: 'a', entryValue: 'a', artist: 'a'}]);
        }
        

        return () => {
            document.removeEventListener("keydown", globalHotkeysEvents.handleKeyDown);
            document.removeEventListener("keyup", globalHotkeysEvents.handleKeyUp);

            globalHotkeysEvents.clear();
        }
    });

    const searchItemRenderer = (searchRes: Search, props: IItemRendererProps) => {
        return (
            <div style={{paddingBottom: props.index === searchResults.length - 1 ? 0 : 10}}>
                <div>{searchRes.entryValue}</div>
                <div style={{fontSize: 12, color: 'rgba(var(--text-secondary), 0.8)'}}>{searchRes.artist === null ? 'Artist' : `${capitalize(searchRes.entryType)} by ${searchRes.artist}`}</div>
            </div>
        );
    }

    const themeEntry = (text: string, isSelected: boolean) => {
        return (
            <div>
                {text}
                {isSelected ? <Icon style={{paddingLeft: 5}} icon='tick'/> : null}
            </div>);
    }
    
    return (
        <>
            <Navbar fixedToTop style={{ height: '40px', paddingRight: 5 }}>
                <NavbarGroup align={Alignment.LEFT} style={{ height: 40, paddingTop: 1 }}>
                    <NavbarHeading style={{ marginRight: 0, marginTop: 4, paddingRight: 7 }}><img src={logo} width={28} height={28} /></NavbarHeading>
                    <NavbarDivider />
                    <Popover autoFocus={false} content={
                        <Menu>
                            <MenuItem icon='cog' text='Settings' onClick={() => setIsOpen(true)} />
                            <MenuItem icon='help' text='Hotkeys' onClick={() =>
                                // Hack to trigger the hotkey menu because sending a keyboard event doesn't set "which" properly
                                globalHotkeysEvents.handleKeyDown({ which: 191, shiftKey: true } as any)
                            } />
                            <MenuItem icon='updated' text='Backup Now' />
                            <MenuItem icon='exchange' text='Switch Theme'>
                                <MenuItem text={themeEntry('Dark', !isLight)} onClick={() => updateTheme('dark')}/>
                                <MenuItem text={themeEntry('Light', isLight)} onClick={() => updateTheme('light')}/>
                            </MenuItem>
                        </Menu>
                    }>
                        <Button minimal icon='menu' />
                    </Popover>

                    <div style={{ width: 5 }} />

                </NavbarGroup>
                <MusicSuggest
                    fill
                    className='search'
                    inputValueRenderer={val => val.entryValue}
                    itemRenderer={searchItemRenderer}
                    onItemSelect={(val, event) => { }}
                    openOnKeyDown={true}
                    items={searchResults}
                    popoverProps={{ minimal: true }}
                    inputProps={{ leftIcon: 'search', rightElement: <Button minimal icon='small-cross' /> }}
                    onQueryChange={async (input, event) => { 
                        await debounced(input);
                    }}
                />
                <MusicOmnibar
                    isOpen={omnibarOpen}
                    itemRenderer={searchItemRenderer}
                    items={searchResults}
                    onItemSelect={(val, event) => { }}
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
                            onClick={() => setSelectedGrid('song')}>
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
                        <Button intent={Intent.SUCCESS} className='hover-intent' style={{ transform: 'translate(0, 1px)' }} onClick={() => {
                            const window = getWindow();
                            if (window?.isMaximized()) {
                                window?.restore();
                            }
                            else {
                                window?.maximize();
                            }
                            window?.reload();
                        }}>
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
}

