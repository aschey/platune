import React, { useState, useEffect, useRef, useCallback, useReducer } from 'react';
import context from '../context';

class AudioQueue {
    switchTime: number;
    index: number;
    finishCounter: number;
    constructor() {
        this.switchTime = 0;
        this.index = 0;
        this.finishCounter = 0;
    }

    findStartGapDuration = (audioBuffer: AudioBuffer) => {
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

    findEndGapDuration = (audioBuffer: AudioBuffer) => {
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

    load = async (song: string, context: AudioContext) => {
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
            startGap: this.findStartGapDuration(audioBuffer),
            endGap: this.findEndGapDuration(audioBuffer)
        }
    }

    schedule = async (song: string, context: AudioContext, onFinished: (playingRow: number) => void, playingRow: number) => {
        const startOffset = this.index === 0 ? 0 : 0; // percentage - this will need to get passed in for pause/resume
        const songData = await this.load(`file://${song}`, context);
        let startSeconds = startOffset > 0 ? Math.round(songData.audioBuffer.duration * startOffset) : songData.startGap;
        let currentSwitchTime = this.switchTime;
        if (this.switchTime === 0) {
            currentSwitchTime = context.currentTime;
        }
        const nextSwitchTime = currentSwitchTime + songData.audioBuffer.duration - startSeconds;
        let start = currentSwitchTime === 0 ? context.currentTime : currentSwitchTime;
        
        songData.source.start(start, startSeconds);
        songData.source.stop(nextSwitchTime);
        let self = this;
        songData.source.addEventListener('ended', function(b) {
            self.finishCounter++;
            if (self.finishCounter % 2 === 0) {
                onFinished(playingRow);
            }
        });
        this.switchTime = nextSwitchTime;
        this.index++;
    }
    
    scheduleAll = async (songQueue: string[], playingRow: number, onFinished: (playingRow: number) => void) => {
        if (songQueue.length) {
            // const newQueue = [
            //     '/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a',
            //     '/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a'
            // ]
            for (let song of songQueue) {
                console.log(song);
                await this.schedule(song, context, onFinished, playingRow);
                playingRow++;
            }
        }
    }
}

export const audioQueue = new AudioQueue();
