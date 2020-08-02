import { sleep } from './util';
//https://codepen.io/anon/pen/aRoVjY

// first song in queue
// if song is more than x bytes: buffer first part of song then switch to full song
// seek location in track: use buffer if loaded or seek location in bounds of first part,
// use https://codepen.io/anon/pen/aRoVjY if mp3 (need to keep loading small chunks if a lot of time remains), otherwise use html5 audio,
// switch back to web audio after song finishes
// still attempt to load song into buffer and determine end gap to subtract

// subsequent songs in queue
// determine if next song can be loaded before current song finishes, if not, use same approach as first song

interface ScheduledSource {
  start: number;
  stop: number;
  source: AudioBufferSourceNode;
  id: number;
}

class AudioQueue {
  switchTime: number;
  index: number;
  finishCounter: number;
  context: AudioContext;
  sources: ScheduledSource[];
  isPaused: boolean;
  html5StartedTime: number;
  audioElement: HTMLAudioElement | null;

  constructor() {
    this.switchTime = 0;
    this.index = 0;
    this.finishCounter = 0;
    this.context = new AudioContext();
    this.sources = [];
    this.isPaused = false;
    this.html5StartedTime = 0;
    this.audioElement = null;
  }

  private findStartGapDuration = (audioBuffer: AudioBuffer) => {
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
  };

  private findEndGapDuration = (audioBuffer: AudioBuffer) => {
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
        return audioBuffer.duration - i / audioBuffer.sampleRate;
      }
    }

    // Hmm, the clip is entirely silent
    return audioBuffer.duration;
  };

  private loadHtml5 = async (song: string) => {
    if (song.endsWith('mp3')) {
      let audio = document.getElementsByTagName('audio')[0];
      audio.src = song;
      this.audioElement = audio;
      audio.addEventListener('canplay', _ => {
        var src = this.context.createMediaElementSource(audio);

        src.connect(this.context.destination);
        audio.play();
      });
      //await audio.play();
    } else {
      let audio = document.getElementsByTagName('audio')[1];
      audio.src = song;
      this.audioElement = audio;
      audio.addEventListener('canplay', _ => {
        this.context.createMediaElementSource(audio).connect(this.context.destination);
        audio.play();
      });
      //await audio.play();
    }
  };

  private load = async (song: string, context: AudioContext) => {
    const data = await fetch(song);
    const arrayBuffer = await data.arrayBuffer();
    const audioBuffer = await context.decodeAudioData(arrayBuffer);
    let gain = this.context.createGain();
    await sleep(3000);
    const source = context.createBufferSource();
    source.connect(gain);
    gain.connect(this.context.destination);
    gain.gain.setValueAtTime(0, this.context.currentTime);
    source.buffer = audioBuffer;
    source.connect(context.destination);
    return {
      audioBuffer,
      source,
      gain,
      startGap: this.findStartGapDuration(audioBuffer),
      endGap: this.findEndGapDuration(audioBuffer),
    };
  };

  private schedule = async (song: string, onFinished: (playingRow: number) => void, playingRow: number) => {
    if (this.index === 0) {
      await this.loadHtml5(`file://${song}`);
      this.html5StartedTime = this.context.currentTime;
    }
    const songData = await this.load(`file://${song}`, this.context);
    let startSeconds = songData.startGap;

    let currentSwitchTime = this.switchTime;
    if (this.switchTime === 0) {
      currentSwitchTime = this.context.currentTime;
    }
    const nextSwitchTime = currentSwitchTime + songData.audioBuffer.duration - startSeconds;
    let start = currentSwitchTime === 0 ? this.context.currentTime : currentSwitchTime;
    if (this.index === 0) {
      let gainVal = 0;

      if (this.audioElement !== null) {
        let gainVal = -1;
        startSeconds = this.context.currentTime - this.html5StartedTime;
        console.log('starting at', startSeconds);

        songData.source.start(start, startSeconds);
        this.audioElement.muted = true;
        return;
        // //songData.source.start(start, startSeconds);
        // while (true) {
        //     await sleep(50);
        //     this.audioElement.volume -= 0.25;
        //     if (gainVal < 0)
        //         gainVal += 0.35;
        //     songData.gain.gain.setValueAtTime(gainVal, this.context.currentTime);
        //     if (this.audioElement.volume < 0.05) {
        //         startSeconds = this.context.currentTime - this.html5StartedTime;
        //         //songData.source.start(start, startSeconds);
        //         //this.audioElement.volume -= 0.01;
        //         this.audioElement?.pause();

        //         this.sources.push({source: songData.source, start, stop: nextSwitchTime, id: playingRow })
        //         let self = this;
        //         songData.source.addEventListener('ended', function(_) {
        //             // don't fire when stopped because we don't want to play the next track (sources will be empty when stopped)
        //             // Sometimes this event fires twice so check the source to ensure we only call onFinished once
        //             if (!self.sources.length || this !== self.sources[0].source) {
        //                 return;
        //             }
        //             // first source in the queue finished, don't need it anymore
        //             self.sources.shift();
        //             onFinished(playingRow);
        //         });
        //         this.switchTime = nextSwitchTime;
        //         this.index++;
        //         break;
        //                     }
        //     }
        //     // startSeconds = this.context.currentTime - this.html5StartedTime;
        //     // console.log('starting at', startSeconds);
        //     // songData.source.start(start, startSeconds);
        //     // while (gainVal < 0) {
        //     //     await sleep(1);
        //     //     gainVal += 0.1;
        //     //     songData.gain.gain.setValueAtTime(gainVal, this.context.currentTime);

        //     // }
        //     songData.source.stop(nextSwitchTime);
      }

      return;
    }
    songData.source.start(start, startSeconds);
    console.log('starting at', startSeconds);
    songData.source.stop(nextSwitchTime);
    this.sources.push({ source: songData.source, start, stop: nextSwitchTime, id: playingRow });
    let self = this;
    songData.source.addEventListener('ended', function (_) {
      // don't fire when stopped because we don't want to play the next track (sources will be empty when stopped)
      // Sometimes this event fires twice so check the source to ensure we only call onFinished once
      if (!self.sources.length || this !== self.sources[0].source) {
        return;
      }
      // first source in the queue finished, don't need it anymore
      self.sources.shift();
      onFinished(playingRow);
    });
    this.switchTime = nextSwitchTime;
    this.index++;
  };

  private reset = () => {
    for (let song of this.sources) {
      song.source.stop();
    }
    this.switchTime = 0;
    this.sources = [];
  };

  private scheduleAll = async (songQueue: string[], playingRow: number, onFinished: (playingRow: number) => void) => {
    if (songQueue.length) {
      let song = songQueue[0];
      //for (let song of songQueue) {
      console.log(song);
      await this.schedule(song, onFinished, playingRow);
      playingRow++;
      //}
    }
  };

  public start = async (songQueue: string[], playingRow: number, onFinished: (playingRow: number) => void) => {
    if (this.isPaused) {
      this.isPaused = false;
      // todo: mute volume while resetting to prevent click
      this.context.resume();
      if (this.sources.length && playingRow !== this.sources[0].id) {
        this.reset();
        return;
      }
    }
    await this.scheduleAll(songQueue, playingRow, onFinished);
  };

  public pause() {
    this.context.suspend();
    this.isPaused = true;
  }

  public stop() {
    this.reset();
  }
}

export const audioQueue = new AudioQueue();
