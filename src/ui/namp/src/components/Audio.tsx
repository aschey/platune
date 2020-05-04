import React, { useState, useEffect } from 'react';

interface AudioProps {
    songQueue: string[]
}

export const Audio: React.FC<AudioProps> = ({songQueue}) => {

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
    
    const load = async (song: string) => {
        const context = new AudioContext();
        const data = await fetch(song);
        const arrayBuffer = await data.arrayBuffer();
        const audioBuffer = await context.decodeAudioData(arrayBuffer);
        const source = context.createBufferSource();
        source.buffer = audioBuffer;
        source.connect(context.destination);
        return {
            context,
            audioBuffer,
            source,
            startGap: findStartGapDuration(audioBuffer),
            endGap: findEndGapDuration(audioBuffer)
        }
    }


    const schedule = async (song1: string, song2: string) => {
        const startOffset = 1; // percentage - this will need to get passed in for pause/resume
        const songData1 = await load(song1);
        const songData2 = await load(song2);
        const startSeconds = Math.round(songData1.audioBuffer.duration * startOffset);
        const switchTime = songData1.audioBuffer.duration - startSeconds;
        songData1.source.start(0, startSeconds);
        songData1.source.stop(switchTime - songData1.endGap);
        songData2.source.start(switchTime, songData2.startGap);
        //songData2.source.stop

    } 

    useEffect(() => {
        const song1 = songQueue.shift();
        const song2 = songQueue.shift();
        if (song1 && song2) {
            schedule(song1, song2);
        }
        
    }, [schedule, load]);

    return <></>
}