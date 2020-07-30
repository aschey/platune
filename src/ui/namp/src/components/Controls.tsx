import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Text, Label, ProgressBar, Intent, Button, Icon, AnchorButton } from '@blueprintjs/core';
import { Slider, Rail, Handles, Tracks, Ticks, SliderItem, GetHandleProps, GetTrackProps } from "react-compound-slider";
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { range, formatMs, sleep } from '../util';
import { SongProgress } from './SongProgress';
import { Volume } from './Volume';
import { shadeColor, hexToRgb, isLight } from '../themes/colorMixer';
import { Song } from '../models/song';
import { audioQueue } from '../audio';
import _ from 'lodash';
import { theme } from './App';

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
    let canvasRef = React.createRef<HTMLCanvasElement>();
    const songColorAdjust = isLight(theme.backgroundMain) ? 150 : -40;

    useEffect(() => {
        visualizer();
    });

    useEffect(() => {
        setColorAdjust(shadeColor(theme.songTimeColor, (progress / songMillis) * songColorAdjust));
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

    const visualizer = async () => {
        if (audioQueue.currentAnalyser && isPlaying) {
            audioQueue.currentAnalyser.fftSize = 2048;
            const bufferLength = audioQueue.currentAnalyser.frequencyBinCount;
            const dataArray = new Uint8Array(bufferLength);
            if (canvasRef.current) {
                const canvasCtx = canvasRef.current.getContext('2d');
                if (!canvasCtx) {
                    return;
                }
                const width = canvasRef.current.width;
                const height = canvasRef.current.height;
                canvasCtx?.clearRect(0, 0, width, height);
                requestAnimationFrame(async () => {
                    audioQueue.currentAnalyser?.getByteFrequencyData(dataArray);
                    const dataLength = _.takeWhile(dataArray, d => d > 0).length;
                    canvasCtx.fillStyle = 'rgba(0,0,0,0)';

                    canvasCtx.fillRect(0, 0, width, height);
                    canvasCtx.strokeStyle = `rgba(${hexToRgb(theme.visualizerColor)}, 0.5)`;
                    canvasCtx.lineWidth = 2;
                    canvasCtx.beginPath();
                    const sliceWidth = width * 1.0 / dataLength;
                    let x = 0;
                    for (var i = 0; i < dataLength; i++) {

                        var v = dataArray[i] / 128.0;
                        var y = v * height / 2;

                        if (i === 0) {
                            canvasCtx.moveTo(x, y);
                        } else {
                            canvasCtx.lineTo(x, y);
                        }

                        x += sliceWidth;
                    }
                    canvasCtx.stroke();
                    await sleep(200);
                    visualizer();
                });
            }

        }
        else {
            await sleep(200);
            visualizer();
        }
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
        <FlexCol className='card visualizer' style={{ marginTop: 5, marginBottom: 5, borderRadius: 10 }}>
            <canvas ref={canvasRef} />
        </FlexCol>
        <FlexCol>
            <FlexRow style={{ fontSize: 16, alignItems: 'center', alignSelf: 'center' }}>
                <div style={{ color: coloradjust }}>{formatMs(progress)}</div>
                <div style={{ color: shadeColor(theme.songTimeColor, songColorAdjust) }}>/{formatMs(songMillis)}</div>
            </FlexRow>
        </FlexCol>
        <FlexRow style={{ fontSize: 16, minWidth: '100%', alignItems: 'center' }}>
            <Icon icon='volume-up' />
            <FlexCol style={{ marginLeft: 10, marginRight: '20%', paddingBottom: 5 }}>
                <Volume />
            </FlexCol>
        </FlexRow>


    </div>;
}