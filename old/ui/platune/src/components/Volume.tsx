import React from 'react';
import { GetTrackProps, Rail, Slider, SliderItem, Tracks } from 'react-compound-slider';
import { audioQueue } from '../audio';
export const Volume: React.FC<{}> = () => {
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
      step={0.05}
      domain={[0, 1]}
      rootStyle={{ position: 'relative', marginBottom: 10, marginTop: 10 }}
      onChange={values => {
        audioQueue.setVolume(values[0]);
      }}
      values={[1] as ReadonlyArray<number>}
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

export const Track: React.FC<ITrackProps> = ({ source, target, getTrackProps }) => (
  <div
    className='volume-slider'
    style={{
      left: `${source.percent}%`,
      width: `${target.percent - source.percent}%`,
    }}
    {...getTrackProps()}
  />
);
