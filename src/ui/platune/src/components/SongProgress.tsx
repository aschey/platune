import React from 'react';
import { Slider, Rail, Handles, Tracks, Ticks, SliderItem, GetHandleProps, GetTrackProps } from 'react-compound-slider';
import { audioQueue } from '../audio';

export const SongProgress: React.FC<{
  songMillis: number;
  progress: number;
  setProgress: (progress: number) => void;
  setPauseTime: (pauseTime: number) => void;
}> = ({ songMillis, progress, setProgress, setPauseTime }) => {
  console.log('millis', songMillis);
  const domain: ReadonlyArray<number> = [0, songMillis];
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

  return (
    <Slider
      mode={1}
      step={1}
      domain={domain}
      rootStyle={sliderStyle}
      onChange={vals => {
        // let val = vals[0];
        // if (val === 0) {
        //   return;
        // }
        // audioQueue.seek(val);
        // setProgress(val);
        // setPauseTime(val);
      }}
      onSlideEnd={vals => {
        console.log(vals);
        let val = vals[0];
        if (val === 0) {
          return;
        }
        audioQueue.seek(val);
        // setProgress(val);
        // setPauseTime(val);
      }}
      values={[progress] as ReadonlyArray<number>}
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
