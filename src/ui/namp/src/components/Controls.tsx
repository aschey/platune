import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Text, Label, ProgressBar, Intent, Button, Icon, AnchorButton } from '@blueprintjs/core';
import { Slider, Rail, Handles, Tracks, Ticks, SliderItem, GetHandleProps, GetTrackProps } from "react-compound-slider";
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { range, formatMs } from '../util';
import { SongProgress } from './SongProgress';
import { Volume } from './Volume';
import { shadeColor } from '../themes/colorMixer';
import { Song } from '../models/song';

interface ControlProps {
    isPlaying: boolean,
    progress: number,
    songMillis: number,
    playingSong: Song | null,
    setIsPlaying: (isPlaying: boolean) => void,
    onPause: () => void,
    onPlay: () => void,
    onStop: () => void
}

export const Controls: React.FC<ControlProps> = ({ isPlaying, setIsPlaying, onPause, onPlay, onStop, progress, songMillis, playingSong }) => {
    let [coloradjust, setColorAdjust] = useState('#000000');
    useEffect(() => {
        setColorAdjust(shadeColor('#92c3e0', -1 * (progress / songMillis) * 40));
    }, [progress, songMillis]);

    const playPauseClick = () => {
        if (isPlaying) {
            onPause();
        }
        else {
            onPlay();
        }
        setIsPlaying(!isPlaying);
    }

    const stopClick = () => {
        onStop();
        setIsPlaying(false);
    }

    return <div style={{ display: 'grid', gridTemplateRows: '10px 60px', gridTemplateColumns: '40% 20% 20% 10% 10%', minHeight: 70 }}>
        <div style={{ gridColumn: '1 / 6' }}>
            <SongProgress songMillis={songMillis} progress={progress} />
        </div>

        <FlexRow style={{ alignItems: 'center', marginLeft: 10 }}>
            {playingSong?.hasArt ? <img src={`http://localhost:5000/albumArt?songId=${playingSong.id}`} width={50} height={50} /> : null}
            <FlexCol style={{ paddingLeft: 10 }}>
                <FlexRow>
                    {playingSong?.name}
                </FlexRow>
                <FlexRow>
                    {playingSong?.album}
                </FlexRow>
                <FlexRow>
                    {playingSong?.artist}
                </FlexRow>
            </FlexCol>

        </FlexRow>
        <FlexCol style={{ alignItems: 'center' }}>
            <FlexRow style={{ alignItems: 'center' }}>
                <Button className='nofocus' intent={Intent.PRIMARY} minimal icon='fast-backward' style={{ borderRadius: '50%', width: 40, height: 40 }} />
                <div style={{ width: 5 }} />
                <Button className='nofocus' intent={isPlaying ? Intent.WARNING : Intent.SUCCESS} minimal icon={isPlaying ? 'pause' : 'play'} style={{ borderRadius: '50%', width: 40, height: 40 }} onClick={playPauseClick} />
                <div style={{ width: 5 }} />
                <Button className='nofocus' intent={Intent.DANGER} minimal icon='stop' style={{ borderRadius: '50%', width: 40, height: 40 }} onClick={stopClick} />
                <div style={{ width: 5 }} />
                <Button className='nofocus' intent={Intent.PRIMARY} minimal icon='fast-forward' style={{ borderRadius: '50%', width: 40, height: 40 }} />
            </FlexRow>

        </FlexCol>
        <FlexCol className='card' style={{ marginTop: 5, marginBottom: 5, background: 'rgba(37, 49, 59, 0.2)', borderRadius: 10 }}>

        </FlexCol>
        <FlexCol>
            <FlexRow style={{ fontSize: 16, alignItems: 'center', alignSelf: 'center' }}>
                <div style={{ color: coloradjust }}>{formatMs(progress)}</div>
                <div style={{ color: shadeColor('#92c3e0', -40) }}>/{formatMs(songMillis)}</div>
            </FlexRow>
        </FlexCol>

        <FlexRow style={{ fontSize: 16, minWidth: '100%', alignItems: 'center' }}>
            <Icon icon='volume-up' />
            <FlexCol style={{ marginLeft: 10, marginRight: 10, paddingBottom: 5 }}>
                <Volume />

            </FlexCol>

        </FlexRow>


    </div>;
}