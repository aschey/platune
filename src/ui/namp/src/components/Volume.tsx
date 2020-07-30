import React from 'react';
import { Slider, Rail, Tracks, SliderItem, GetTrackProps } from 'react-compound-slider';
export const Volume: React.FC<{}> = () => {
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
    return <Slider
        mode={1}
        step={1}
        domain={[0, 100]}
        rootStyle={sliderStyle}
        onChange={(a) => { }}
        values={[0] as ReadonlyArray<number>}
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
}
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

