import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Text, Label, ProgressBar, Intent, Button, Icon, AnchorButton } from '@blueprintjs/core';
import { Slider, Rail, Handles, Tracks, Ticks, SliderItem, GetHandleProps, GetTrackProps } from "react-compound-slider";
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { range } from '../util';

interface ControlProps {
    isPlaying: boolean,
    progress: number,
    songMillis: number,
    setIsPlaying: (isPlaying: boolean) => void,
    onPause: () => void,
    onPlay: () => void,
    onStop: () => void
}

export const Controls: React.FC<ControlProps> = ({ isPlaying, setIsPlaying, onPause, onPlay, onStop, progress, songMillis }) => {
    const domain: ReadonlyArray<number> = [0, songMillis];
    const sliderStyle: React.CSSProperties = {
        position: 'relative',
        marginBottom: 10,
        marginTop: 10
    };
    const railStyle: React.CSSProperties = {
        position: 'absolute',
        width: '100%',
        height: 5,
        borderRadius: 7,
        cursor: 'pointer',
        zIndex: 1,
        boxShadow: '0 -1px 1px rgba(0, 0, 0, 0.4)',
        backgroundColor: 'rgb(155,155,155)'
    };

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

    return (
        <>
            <FlexRow style={{ alignItems: 'center', paddingTop: 0, minWidth: '100%' }}>
                <FlexCol style={{ minWidth: '100%' }}>
                    <FlexCol style={{ width: '100%', minHeight: 20, maxHeight: 20, transform: 'translate(0, -6px)' }}>
                        <Slider
                            mode={1}
                            step={1}
                            domain={domain}
                            rootStyle={sliderStyle}
                            onChange={(a) => { }}
                            values={[progress] as ReadonlyArray<number>}
                        >
                            <Rail>
                                {({ getRailProps }) => (
                                    <div style={railStyle} {...getRailProps()} />
                                )}
                            </Rail>
                            <Tracks right={false}>
                                {({ tracks, getTrackProps }) => (
                                    <div className="slider-tracks">
                                        {tracks.map(({ id, source, target }) => (
                                            <Track
                                                key={id}
                                                source={source}
                                                target={target}
                                                getTrackProps={getTrackProps}
                                            />
                                        ))}
                                    </div>
                                )}
                            </Tracks>
                        </Slider>
                    </FlexCol>
                    <FlexRow style={{ position: 'absolute', bottom: 0, height: 60 }}>
                        <img src="http://localhost:5000/albumArt?songId=2" width={60} height={60} />
                        <FlexCol>
                            <FlexRow>
                                song
                            </FlexRow>
                            <FlexRow>
                                album
                            </FlexRow>
                            <FlexRow>
                                artist
                            </FlexRow>
                        </FlexCol>
                    </FlexRow>

                    <FlexRow>
                        <FlexCol>

                        </FlexCol>
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
                        <FlexCol>
                            <FlexRow>
                                <FlexCol className='card' style={{ minHeight: 40, minWidth: 200, background: 'rgba(37, 49, 59, 0.2)', borderRadius: 10, marginRight: 10 }}></FlexCol>
                                <FlexCol style={{ alignItems: 'center' }}>
                                    <div>2:00/4:00</div>
                                    <div>volume</div>
                                </FlexCol>
                            </FlexRow>

                        </FlexCol>
                    </FlexRow>

                </FlexCol>

            </FlexRow>
        </>
    );

}

interface IHandleProps {
    domain: ReadonlyArray<number>;
    handle: SliderItem;
    getHandleProps: GetHandleProps;
}

export const Handle: React.FC<IHandleProps> = ({
    domain: [min, max],
    handle: { id, value, percent },
    getHandleProps
}) => (
        <div
            role="slider"
            aria-valuemin={min}
            aria-valuemax={max}
            aria-valuenow={value}
            style={{
                left: `${percent}%`,
                position: 'absolute',
                marginLeft: '-11px',
                marginTop: '-3px',
                zIndex: 2,
                width: 10,
                height: 10,
                cursor: 'pointer',
                borderRadius: '50%',
                boxShadow: '1px 1px 1px 1px rgba(0, 0, 0, 0.2)',
                backgroundColor: '#34568f'
            }}
            {...getHandleProps(id)}
        />
    );

// *******************************************************
// TRACK COMPONENT
// *******************************************************
interface ITrackProps {
    source: SliderItem;
    target: SliderItem;
    getTrackProps: GetTrackProps;
}
//let val = `linear-gradient(to right, ${range(100).map(val => `rgba(${Math.round(Math.random() * 100)},${Math.round(Math.random() * 100)},${Math.round(Math.random() * 100)},1) ${val}%`).join(',')}) fixed`;
export const Track: React.FC<ITrackProps> = ({
    source,
    target,
    getTrackProps
}) => (
        <div
            style={{
                position: 'absolute',
                height: 5,
                zIndex: 1,
                background: `linear-gradient(to right, rgba(25,94,145,1) 0%, rgba(20,186,142,1) 100%) fixed`,
                borderRadius: 7,
                cursor: 'pointer',
                left: `${source.percent}%`,
                width: `${target.percent - source.percent}%`
            }}
            {...getTrackProps()}
        />
    );