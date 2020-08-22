import React, { useState, useEffect, useRef } from 'react';
import {
  Slider,
  Rail,
  Handles,
  Tracks,
  Ticks,
  SliderItem,
  GetHandleProps,
  GetTrackProps,
  GetRailProps,
} from 'react-compound-slider';
import { audioQueue } from '../audio';
import { useObservable } from 'rxjs-hooks';

export const SongProgress: React.FC<{}> = () => {
  const songMillis = useObservable(() => audioQueue.durationMillis);
  const progress = useObservable(() => audioQueue.progress);
  const [lastProgress, setLastProgress] = useState<ReadonlyArray<number>>([0]);
  const isSeeking = useRef(false);
  const sliderStyle: React.CSSProperties = {
    position: 'relative',
    marginTop: 5,
  };
  const railStyle: React.CSSProperties = {
    position: 'absolute',
    width: '100%',
    height: 5,
    borderRadius: 7,
    cursor: 'pointer',
    zIndex: 1,
    boxShadow: '0 -1px 1px rgba(0, 0, 0, 0.4)',
    backgroundColor: 'rgb(155,155,155)',
  };

  useEffect(() => {
    if (!isSeeking.current) {
      setLastProgress([progress ?? 0]);
    }
  }, [progress]);

  const onSlideStart = () => {
    console.log('seeking');
    isSeeking.current = true;
  };

  return (
    <Slider
      mode={1}
      step={1}
      domain={[0, songMillis ?? 0]}
      rootStyle={{ position: 'relative', marginTop: 5 }}
      onSlideStart={onSlideStart}
      onSlideEnd={vals => {
        let val = vals[0];
        if (val === 0) {
          return;
        }
        audioQueue.seek(val);
        isSeeking.current = false;
      }}
      values={lastProgress}
    >
      <Rail>{({ getRailProps }) => <div style={railStyle} {...getRailProps()} />}</Rail>
      <Tracks right={false}>
        {({ tracks, getTrackProps }) => (
          <div className='slider-tracks'>
            {tracks.map(({ id, source, target }) => (
              <Track key={id} source={source} target={target} getTrackProps={getTrackProps} />
            ))}
          </div>
        )}
      </Tracks>
    </Slider>
  );
};

interface ITrackProps {
  source: SliderItem;
  target: SliderItem;
  getTrackProps: GetTrackProps;
}

//let val = `linear-gradient(to right, ${range(100).map(val => `rgba(${Math.round(Math.random() * 100)},${Math.round(Math.random() * 100)},${Math.round(Math.random() * 100)},1) ${val}%`).join(',')}) fixed`;
export const Track: React.FC<ITrackProps> = ({ source, target, getTrackProps }) => (
  <div
    className='song-progress-track'
    style={{
      left: `${source.percent}%`,
      width: `${target.percent - source.percent}%`,
    }}
    {...getTrackProps()}
  />
);
