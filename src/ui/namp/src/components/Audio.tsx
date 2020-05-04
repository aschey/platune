import React, { useState, useEffect, useRef, useCallback } from 'react';
import context from '../context';

interface AudioProps {
    songQueue: string[],
    onFinished: () => void
}

export const Audio: React.FC<AudioProps> = ({songQueue, onFinished}) => {
    const switchTime = useRef<number>(0);
    const index = useRef<number>(0);
    const finishCounter = useRef<number>(0);

    const findStartGapDuration = (audioBuffer: AudioBuffer) => {
        // Get the raw audio data for the left & right channels.
        const l = audioBuffer.getChannelData(0);
        const r = audioBuffer.getChannelData(1);
        // Each is an array of numbers between -1 and 1 describing
        // the waveform, sample by sample.

        // Now to figure out how long both channels remain at 0:
        for (let i = 0; i < l.length; i++) {
            if (l[i] || r[i]) {
                // Now we know which sample is non-zero, but we want
                // the gap in seconds, not samples. Thankfully sampleRate
                // gives us the number of samples per second.
                return i / audioBuffer.sampleRate;
            }
        }

        // Hmm, the clip is entirely silent
        return audioBuffer.duration;
    }

    const findEndGapDuration = (audioBuffer: AudioBuffer) => {
        // Get the raw audio data for the left & right channels.
        const l = audioBuffer.getChannelData(0);
        const r = audioBuffer.getChannelData(1);
        // Each is an array of numbers between -1 and 1 describing
        // the waveform, sample by sample.

        // Now to figure out how long both channels remain at 0:
        for (let i = l.length - 1; i >= 0; i--) {
            if (l[i] || r[i]) {
                // Now we know which sample is non-zero, but we want
                // the gap in seconds, not samples. Thankfully sampleRate
                // gives us the number of samples per second.
                return audioBuffer.duration - (i / audioBuffer.sampleRate);
            }
        }

        // Hmm, the clip is entirely silent
        return audioBuffer.duration;
    }
    
    const load = useCallback(async (song: string, context: AudioContext) => {
        //const context = new AudioContext();
        const data = await fetch(song);
        const arrayBuffer = await data.arrayBuffer();
        const audioBuffer = await context.decodeAudioData(arrayBuffer);
        const source = context.createBufferSource();
        source.buffer = audioBuffer;
        source.connect(context.destination);
        return {
            //context,
            audioBuffer,
            source,
            startGap: findStartGapDuration(audioBuffer),
            endGap: findEndGapDuration(audioBuffer)
        }
    }, []);

    const schedule = useCallback(async (song: string, context: AudioContext) => {
        console.log('here');
        const startOffset = index.current === 0 ? 0.0 : 0; // percentage - this will need to get passed in for pause/resume
        const songData = await load(`file://${song}`, context);
        let startSeconds = startOffset > 0 ? Math.round(songData.audioBuffer.duration * startOffset) : songData.startGap;
        if (switchTime.current === 0) {
            switchTime.current = context.currentTime;
        }
        const nextSwitchTime = switchTime.current + songData.audioBuffer.duration - startSeconds;
        let start = switchTime.current === 0 ? context.currentTime : switchTime.current;
        
        songData.source.start(start, startSeconds);
        console.log('start' + index.current, start, startSeconds);
        songData.source.stop(nextSwitchTime);
        console.log('stop' + index.current, nextSwitchTime);
        songData.source.addEventListener('ended', function(b) {
            console.log('ended');
            finishCounter.current++;
            if (finishCounter.current % 2 === 1) {
                onFinished();
            }
        });
        
        switchTime.current = nextSwitchTime;
        index.current++;
    }, [load, onFinished]);

    useEffect(() => {        
        const scheduleAll = async () => {
            console.log('test');
            if (songQueue.length) {
                // const newQueue = [
                //     '/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a',
                //     '/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a'
                // ]
                const newQueue = songQueue.slice(0);
                while (newQueue.length) {
                    const song = newQueue.shift();
                    console.log(song);
                    if (song) {
                        await schedule(song, context);
                    }
                }
            }
        }
        scheduleAll();
    }, [schedule, load, songQueue]);

    return <></>
}