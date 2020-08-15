import { sleep } from './util';
import { throws } from 'assert';

interface ScheduledSource {
  start: number;
  stop: number;
  source: AudioNodeWrapper;
  analyser: AnalyserNode;
  gain: GainNode;
  id: number;
}

interface AudioMetadata {
  audioNode: AudioNodeWrapper;
  analyser: AnalyserNode;
  gain: GainNode;
  startGap: number;
  endGap: number;
}

interface HtmlAudioMetadata {
  element: HTMLAudioElement;
  audioNode: MediaElementAudioSourceNode;
  gain: GainNode;
  analyser: AnalyserNode;
}

class AudioNodeWrapper {
  bufferNode: AudioBufferSourceNode | null;
  htmlNode: HTMLAudioElement | null;
  context: AudioContext;
  startGap: number;
  endGap: number;

  constructor(node: AudioBufferSourceNode | HTMLAudioElement, startGap: number, endGap: number, context: AudioContext) {
    if (node instanceof HTMLAudioElement) {
      this.htmlNode = node;
      this.bufferNode = null;
    } else {
      this.bufferNode = node;
      this.htmlNode = null;
    }
    this.context = context;
    this.startGap = startGap;
    this.endGap = endGap;
  }

  public duration = () => (this.htmlNode ? this.htmlNode?.duration : this.bufferNode?.buffer?.duration) ?? 0;

  public async start(when: number, duration: number) {
    if (this.htmlNode) {
      this.htmlNode.currentTime = when - this.context.currentTime;
      await this.htmlNode?.play();
    } else {
      this.bufferNode?.start(when, duration);
    }
  }

  public stop(when: number) {
    if (this.bufferNode) {
      this.bufferNode?.stop(when);
    }
  }

  public seek(millis: number) {
    const startSeconds = millis / 1000;
    if (this.htmlNode) {
      this.htmlNode.currentTime = startSeconds;
    } else if (this.bufferNode?.buffer) {
      this.bufferNode.stop();
      const source = this.context.createBufferSource();
      const analyser = this.context.createAnalyser();
      const gain = this.context.createGain();
      source.buffer = this.bufferNode.buffer;
      source.connect(analyser);
      source.connect(gain);
      gain.connect(this.context.destination);
      gain.gain.value = audioQueue.volume;
      this.bufferNode = source;
      debugger;
      this.bufferNode.start(this.context.currentTime, startSeconds);
      this.bufferNode.stop(this.context.currentTime + source.buffer.duration - startSeconds - this.endGap);
    }
  }

  public stopNow() {
    if (this.htmlNode) {
      this.htmlNode.pause();
      this.htmlNode.currentTime = 0;
      this.htmlNode.src = '';
    } else {
      this.bufferNode?.stop();
    }
  }

  public onEnded(handler: (current: AudioNodeWrapper) => void) {
    if (this.htmlNode) {
      this.htmlNode.onended = () => handler(this);
    } else {
      this.bufferNode?.addEventListener('ended', () => handler(this));
    }
  }
}

class AudioQueue {
  switchTime: number;
  index: number;
  finishCounter: number;
  context: AudioContext;
  sources: ScheduledSource[];
  isPaused: boolean;
  currentAnalyser: AnalyserNode | null;
  volume: number;
  private currentGain: GainNode | null;
  private htmlAudio: HtmlAudioMetadata;

  constructor() {
    this.switchTime = 0;
    this.index = 0;
    this.finishCounter = 0;
    this.context = new AudioContext();
    this.sources = [];
    this.isPaused = false;
    this.currentAnalyser = null;
    this.currentGain = null;
    this.volume = 1;
    const audioElement = document.getElementsByTagName('audio')[0];
    this.htmlAudio = {
      element: audioElement,
      audioNode: this.context.createMediaElementSource(audioElement),
      gain: this.context.createGain(),
      analyser: this.context.createAnalyser(),
    };
    this.htmlAudio.audioNode.connect(this.htmlAudio.analyser);
    this.htmlAudio.audioNode.connect(this.htmlAudio.gain);
    this.htmlAudio.gain.connect(this.context.destination);
  }

  private findStartGapDuration = (audioBuffer: AudioBuffer) => {
    // Get the raw audio data for the left & right channels.
    const l = audioBuffer.getChannelData(0);
    const r = audioBuffer.numberOfChannels > 1 ? audioBuffer.getChannelData(1) : l;
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
    const r = audioBuffer.numberOfChannels > 1 ? audioBuffer.getChannelData(1) : l;
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
    const element = this.htmlAudio.element;
    element.src = song;
    const promise = new Promise<AudioMetadata>((resolve, reject) => {
      element.onloadedmetadata = () => {
        this.htmlAudio.gain.gain.value = this.volume;
        resolve({
          audioNode: new AudioNodeWrapper(element, 0, 0, this.context),
          analyser: this.htmlAudio.analyser,
          gain: this.htmlAudio.gain,
          startGap: 0,
          endGap: 0,
        });
      };
    });

    return promise;
  };

  private load = async (song: string): Promise<AudioMetadata> => {
    const data = await fetch(song);
    const arrayBuffer = await data.arrayBuffer();
    // Todo: cache audio buffers if they are large enough to warrant caching
    const audioBuffer = await this.context.decodeAudioData(arrayBuffer);
    const source = this.context.createBufferSource();
    const analyser = this.context.createAnalyser();
    const gain = this.context.createGain();
    source.buffer = audioBuffer;
    source.connect(analyser);
    source.connect(gain);
    gain.connect(this.context.destination);
    gain.gain.value = this.volume;
    const startGap = this.findStartGapDuration(audioBuffer);
    const endGap = this.findEndGapDuration(audioBuffer);
    return {
      audioNode: new AudioNodeWrapper(source, startGap, endGap, this.context),
      analyser,
      gain,
      startGap,
      endGap,
    };
  };

  private schedule = async (song: string, onFinished: (playingRow: number) => void, playingRow: number) => {
    //const startOffset = this.sources.length === 0 ? 0.97 : 0;
    const songData = await (this.sources.length === 0 ? this.load(`file://${song}`) : this.load(`file://${song}`));
    let startSeconds = songData.startGap;
    let currentSwitchTime = this.switchTime;
    if (this.switchTime === 0) {
      currentSwitchTime = this.context.currentTime;
    }
    if (this.sources.length === 0) {
      audioQueue.currentAnalyser = songData.analyser;
      audioQueue.currentGain = songData.gain;
    }
    let nextSwitchTime = currentSwitchTime + songData.audioNode.duration() - startSeconds;
    if (this.sources.length === 0) {
      nextSwitchTime -= 0.1;
    }
    let start = currentSwitchTime === 0 ? this.context.currentTime : currentSwitchTime;
    await songData.audioNode.start(start, startSeconds);
    console.log('starting at', startSeconds);
    songData.audioNode.stop(nextSwitchTime);
    this.sources.push({
      source: songData.audioNode,
      analyser: songData.analyser,
      gain: songData.gain,
      start,
      stop: nextSwitchTime,
      id: playingRow,
    });
    let self = this;
    songData.audioNode.onEnded((current: AudioNodeWrapper) => {
      // don't fire when stopped because we don't want to play the next track (sources will be empty when stopped)
      // Sometimes this event fires twice so check the source to ensure we only call onFinished once
      if (!self.sources.length || current !== self.sources[0].source) {
        return;
      }
      // first source in the queue finished, don't need it anymore
      self.sources.shift();
      if (self.sources.length) {
        self.currentAnalyser = self.sources[0].analyser;
        self.currentGain = self.sources[0].gain;
      }
      onFinished(playingRow);
    });
    this.switchTime = nextSwitchTime;
    this.index++;
  };

  private reset = () => {
    for (let song of this.sources) {
      song.source.stopNow();
    }
    this.switchTime = 0;
    this.sources = [];
    if (this.currentGain) {
      this.currentGain.gain.value = 0;
    }
  };

  private scheduleAll = async (songQueue: string[], playingRow: number, onFinished: (playingRow: number) => void) => {
    if (songQueue.length) {
      for (let song of songQueue) {
        console.log(song);
        await this.schedule(song, onFinished, playingRow);
        playingRow++;
      }
    }
  };

  public start = async (songQueue: string[], playingRow: number, onFinished: (playingRow: number) => void) => {
    if (this.isPaused) {
      this.isPaused = false;
      this.setVolumeTemporary(this.volume);
      this.context.resume();
      // Starting the song that's currently paused, don't reschedule
      if (this.sources.length && playingRow === this.sources[0].id) {
        return;
      } else {
        this.reset();
      }
    }
    await this.scheduleAll(songQueue, playingRow, onFinished);
  };

  private setVolumeTemporary(volume: number) {
    if (this.currentGain) {
      this.currentGain.gain.value = volume;
    }
    this.sources.forEach(s => (s.gain.gain.value = volume));
  }

  public setVolume(volume: number) {
    this.setVolumeTemporary(volume);
    this.volume = volume;
  }

  public async pause() {
    this.setVolumeTemporary(0);
    // Seems like there's a slight delay before the volume change takes so we have to wait for it to complete
    await sleep(100);
    await this.context.suspend();
    this.isPaused = true;
  }

  public stop() {
    this.reset();
  }

  public seek(millis: number) {
    if (this.sources.length) {
      this.sources[0].source.seek(millis);
    }
  }
}

export const audioQueue = new AudioQueue();
