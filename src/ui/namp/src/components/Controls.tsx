import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Text, Label, ProgressBar, Intent, Button, Icon } from '@blueprintjs/core';
import { Slider, Rail, Handles, Tracks, Ticks, SliderItem, GetHandleProps, GetTrackProps } from "react-compound-slider";
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';

export const Controls: React.FC<{}> = () => {
    const domain: ReadonlyArray<number> = [100, 500];
    const sliderStyle: React.CSSProperties = {
        margin: '5%',
        position: 'relative',
        width: '90%',
        marginBottom: 10,
        marginTop: 10
      };
      const railStyle: React.CSSProperties = {
        position: 'absolute',
        width: '100%',
        height: 5,
        borderRadius: 7,
        cursor: 'pointer',
        backgroundColor: 'rgb(155,155,155)'
      };
      
    return (
        <FlexRow style={{height: 80, alignItems: 'center'}}>
            <FlexCol style={{alignItems: 'center', border: '1px solid #263540', boxShadow: '1px 1px #263540', background: '#334654', borderRadius: '10px', marginLeft: 30, marginRight: 30}}>
                <FlexRow style={{alignItems: 'center', paddingTop: 5}}>
                    <Button intent={Intent.PRIMARY} outlined icon='fast-backward' style={{borderRadius: '50%', width: 30, height: 30}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.WARNING} outlined icon='pause' style={{borderRadius: '50%', width: 35, height: 35}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.SUCCESS} outlined icon='play' style={{borderRadius: '50%', width: 40, height: 40}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.DANGER} outlined icon='stop' style={{borderRadius: '50%', width: 35, height: 35}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.PRIMARY} outlined icon='fast-forward' style={{borderRadius: '50%', width: 30, height: 30}}/>
                </FlexRow>
                <FlexRow  style={{width: window.innerWidth * .8}}>
                <Slider
                    mode={1}
                    step={1}
                    domain={domain}
                    rootStyle={sliderStyle}
                    onChange={(a) => {}}
                    values={[150] as ReadonlyArray<number>}
                    >
                    <Rail>
                        {({ getRailProps }) => (
                        <div style={railStyle} {...getRailProps()} />
                        )}
                    </Rail>
                    <Handles>
                        {({ handles, getHandleProps }) => (
                        <div className="slider-handles">
                            {handles.map(handle => (
                            <Handle
                                key={handle.id}
                                handle={handle}
                                domain={domain}
                                getHandleProps={getHandleProps}
                            />
                            ))}
                        </div>
                        )}
                        </Handles>
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
                </FlexRow>      
            </FlexCol>
        </FlexRow>
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