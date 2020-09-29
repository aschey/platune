import { Button, Icon, Intent, Text } from '@blueprintjs/core';
import _ from 'lodash';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useObservable } from 'rxjs-hooks';
import { audioQueue, PlaybackState } from '../audio';
import { Song } from '../models/song';
import { useThemeContext } from '../state/themeContext';
import { hexToRgb, isLight, shadeColor } from '../themes/colorMixer';
import { formatMs } from '../util';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { SongProgress } from './SongProgress';
import { Volume } from './Volume';

interface ControlProps {
  playingSong: Song | null;
  onPlay: () => Promise<void>;
  onPrevious: () => Promise<void>;
}

export const Controls: React.FC<ControlProps> = ({ onPlay, onPrevious, playingSong }) => {
  const [coloradjust, setColorAdjust] = useState('#000000');
  const songMillis = useObservable(() => audioQueue.durationMillis);
  const progress = useObservable(() => audioQueue.progress);
  const playbackState = useObservable(() => audioQueue.playbackState);
  const canvasRef = React.createRef<HTMLCanvasElement>();
  const visualizerTimeout = useRef<NodeJS.Timeout>();
  const { isLightTheme, themeVal } = useThemeContext();

  const songColorAdjust = isLightTheme ? 150 : -40;

  const stopVisualizer = useCallback(() => {
    if (visualizerTimeout.current) {
      clearTimeout(visualizerTimeout.current);
    }
  }, []);

  const visualizer = useCallback(async () => {
    if (audioQueue.currentAnalyser && playbackState === PlaybackState.Playing) {
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
          canvasCtx.strokeStyle = `rgba(${hexToRgb(themeVal.visualizerColor)}, 0.5)`;
          canvasCtx.lineWidth = 2;
          canvasCtx.beginPath();
          const sliceWidth = (width * 1.0) / dataLength;
          let x = 0;
          for (var i = 0; i < dataLength; i++) {
            var v = dataArray[i] / 128.0;
            var y = (v * height) / 2;

            if (i === 0) {
              canvasCtx.moveTo(x, y);
            } else {
              canvasCtx.lineTo(x, y);
            }

            x += sliceWidth;
          }
          canvasCtx.stroke();
          visualizerTimeout.current = setTimeout(visualizer, 50);
        });
      }
    } else {
      visualizerTimeout.current = setTimeout(visualizer, 50);
    }
  }, [canvasRef, playbackState]);

  useEffect(() => {
    if (audioQueue.isPlaying()) {
      visualizer();
    } else {
      stopVisualizer();
    }
    return stopVisualizer;
  }, [playbackState, visualizer, stopVisualizer]);

  useEffect(() => {
    if (songMillis !== null && progress !== null) {
      setColorAdjust(shadeColor(themeVal.songTimeColor, (progress / songMillis) * songColorAdjust));
    }
  }, [progress, songMillis, songColorAdjust]);

  const playPauseClick = async () => {
    if (audioQueue.isPlaying()) {
      await audioQueue.pause();
    } else {
      await onPlay();
    }
  };

  return (
    <div style={{ gridColumn: '1 / 3' }}>
      <div
        style={{
          display: 'grid',

          gridTemplateRows: '12px 58px',
          gridTemplateColumns: window.innerWidth > 1600 ? `5fr 175px 4fr 1fr` : `5fr 175px 3fr 2fr`,
        }}
      >
        <div style={{ gridColumn: '1 / 6' }}>
          <SongProgress />
        </div>

        <FlexRow style={{ marginLeft: 10, overflow: 'hidden' }}>
          {playingSong?.hasArt ? (
            <img
              src={`http://localhost:5000/albumArt?songId=${playingSong.id}`}
              alt='current song artwork'
              width={50}
              height={50}
            />
          ) : null}
          <div style={{ paddingLeft: 10, overflow: 'hidden' }}>
            <FlexRow>
              <Text ellipsize>{playingSong?.name}</Text>
            </FlexRow>
            <FlexRow>
              <Text ellipsize>{playingSong?.album}</Text>
            </FlexRow>
            <FlexRow>
              <Text ellipsize>{playingSong?.artist}</Text>
            </FlexRow>
          </div>
          <FlexCol>
            {(songMillis ?? 0) > 0 ? (
              <FlexRow style={{ fontSize: 16, paddingLeft: 10, paddingRight: 10 }}>
                <div style={{ color: coloradjust }}>{formatMs(progress ?? 0)}</div>
                <div style={{ color: shadeColor(themeVal.songTimeColor, songColorAdjust) }}>
                  /{formatMs(songMillis ?? 0)}
                </div>
              </FlexRow>
            ) : null}
          </FlexCol>
        </FlexRow>

        <FlexCol>
          <FlexRow>
            <Button
              className='nofocus'
              intent={Intent.PRIMARY}
              minimal
              icon='fast-backward'
              style={{ borderRadius: '50%', width: 40, height: 40 }}
              onClick={onPrevious}
            />
            <div style={{ width: 5 }} />
            <Button
              className='nofocus'
              intent={playbackState === PlaybackState.Playing ? Intent.WARNING : Intent.SUCCESS}
              minimal
              icon={playbackState === PlaybackState.Playing ? 'pause' : 'play'}
              style={{ borderRadius: '50%', width: 40, height: 40 }}
              onClick={playPauseClick}
            />
            <div style={{ width: 5 }} />
            <Button
              className='nofocus'
              intent={Intent.DANGER}
              minimal
              icon='stop'
              style={{ borderRadius: '50%', width: 40, height: 40 }}
              onClick={() => audioQueue.stop()}
            />
            <div style={{ width: 5 }} />
            <Button
              className='nofocus'
              intent={Intent.PRIMARY}
              minimal
              icon='fast-forward'
              style={{ borderRadius: '50%', width: 40, height: 40 }}
              onClick={audioQueue.next}
            />
          </FlexRow>
        </FlexCol>

        <FlexCol
          className='card visualizer'
          style={{ marginTop: 7, marginBottom: 7, marginLeft: '10%', marginRight: '10%', borderRadius: 10 }}
        >
          <canvas ref={canvasRef} />
        </FlexCol>
        <FlexRow style={{ fontSize: 16 }}>
          <Icon icon='volume-up' />
          <FlexCol
            center={false}
            style={{
              alignSelf: 'center',
              alignContent: 'center',
              marginLeft: 10,
              marginRight: '20%',
              paddingBottom: 4,
            }}
          >
            <Volume />
          </FlexCol>
        </FlexRow>
      </div>
    </div>
  );
};
