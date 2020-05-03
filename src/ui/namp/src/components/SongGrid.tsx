import React, { useState, useEffect, useRef } from 'react';
import { Table, Cell, Column, SelectionModes, IRegion } from '@blueprintjs/table';
import { Text, Label } from '@blueprintjs/core';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { range, sleep } from '../util';
var Sound = require('react-sound').default;

declare global {
    interface Window { soundManager: any; }
}

export const SongGrid: React.FC<{}> = () => {
    const [songs, setSongs] = useState<Song[]>([]);
    const [, setState] = useState();
    const [selectedRow, setSelectedRow] = useState<number>(-1);
    const [testText, setTestText] = useState<string>('test');
    const rowRef = useRef<number>(-1);
    const numTries = 10;

    const loadSongs = async () => {
        for (let i of range(numTries)) {
            try {
                console.log(i);
                const songs = await getJson<Song[]>('/songs');
                return songs;
            }
            catch (e) {
                if (i === numTries - 1) {
                    throw e;
                }
                await sleep(1000);
            }
        }
        return [];
    }

    useEffect(() => {
        //setInterval(() => console.log('test'), 1000);
       // var gapless5 = require('../gapless5').default;
        //console.log(gapless5);
        // var player = new gapless5("gapless5-block", {
        //     tracks: ["loop1.mp3", "loop2.mp3"],
        //     loop: true,
        //     playOnLoad: true,
        //     mapKeys: {prev: "a", playpause: "s", stop: "d", next: "f"}
        //   });
        loadSongs().then(setSongs);
    }, []);

    const titleRenderer = (rowIndex: number) => {
        console.log('here');
        return (
            <Cell>
                <Text>
                    {songs[rowIndex].name}
                </Text>
                <Sound
                    autoLoad={false}
                    volume={30}
                    url={`file://${songs[rowIndex].path}`}
                    playStatus={rowIndex === rowRef.current ? Sound.status.PLAYING : Sound.status.STOPPED}
                    onFinishedPlaying={() => {
                        console.log('here');
                        console.log(selectedRow);
                        rowRef.current = rowRef.current + 1;
                        setSelectedRow(selectedRow + 1);
                        setState({});
                        window.soundManager.play("sound" + (rowRef.current));
                    }
                    }
                />
            </Cell>
        )
    }
    return (
        <>
        <Label>{testText}</Label>
        <Table numRows={songs.length} selectionModes={SelectionModes.ROWS_AND_CELLS} forceRerenderOnSelectionChange={true} selectedRegionTransform={(region, event) => ({rows: region.rows})} onSelection={(p: IRegion[]) => {
                if (p.length > 0 && p[0] !== null && p[0] !== undefined && p[0].rows !== null && p[0].rows !== undefined) {
                    setSelectedRow(p[0].rows[0]);
                    rowRef.current = p[0].rows[0];
                }
            }
            }>
            <Column name='title' cellRenderer={titleRenderer}/>
            <Column name='album artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].albumArtist}</Text></Cell>}/>
            <Column name='artist' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].artist}</Text></Cell>}/>
            <Column name='album' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].album}</Text></Cell>}/>
            <Column name='path' cellRenderer={(rowIndex) => <Cell><Text>{songs[rowIndex].path}</Text></Cell>}/>
        </Table>
        </>
    )
}