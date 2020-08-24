import { sleep } from './util';
import { Subject, interval, Observable } from 'rxjs';
import { map } from 'rxjs/operators';

interface ScheduledSource {
  start: number;
  stop: number;
  file: string;
  source: AudioNodeWrapper;
  analyser: AnalyserNode;
  gain: GainNode;
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

  public async start(when: number, offset: number) {
    if (this.htmlNode) {
      this.htmlNode.currentTime = offset;
      await this.htmlNode?.play();
    } else {
      this.bufferNode?.start(when, offset);
    }
  }

  public stop(when: number) {
    if (this.bufferNode) {
      this.bufferNode?.stop(when);
    }
  }

  public seek(millis: number) {
    const startSeconds = millis / 1000;
    const sources = audioQueue.sources;
    const unscheduled = audioQueue.unscheduled;
    audioQueue.start(sources.map(s => s.file).concat(unscheduled), startSeconds);
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

  public onEnded(handler: (current: AudioNodeWrapper) => Promise<void>) {
    if (this.htmlNode) {
      this.htmlNode.onended = async () => await handler(this);
    } else {
      this.bufferNode?.addEventListener('ended', () => handler(this));
    }
  }
}

class AudioQueue {
  switchTime: number;
  //index: number;
  finishCounter: number;
  context: AudioContext;
  sources: ScheduledSource[];
  isPaused: boolean;
  isPlaying: boolean;
  isPlayingEvent: Subject<boolean>;
  playingSource: Subject<string>;
  durationMillis: Subject<number>;
  progress: Observable<number>;
  currentAnalyser: AnalyserNode | null;
  volume: number;
  startTime: number;
  private currentGain: GainNode | null;
  private htmlAudio: HtmlAudioMetadata;
  unscheduled: string[];

  constructor() {
    this.switchTime = 0;
    this.finishCounter = 0;
    this.context = new AudioContext();
    this.sources = [];
    this.isPaused = false;
    this.isPlaying = false;
    this.currentAnalyser = null;
    this.currentGain = null;
    this.volume = 1;
    this.durationMillis = new Subject<number>();
    this.playingSource = new Subject<string>();
    this.startTime = 0;
    this.progress = interval(200).pipe(map(this.getCurrentTime));
    this.unscheduled = [];
    this.isPlayingEvent = new Subject<boolean>();

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

  private getCurrentTime = () => {
    return this.isPlaying || this.isPaused ? this.context.currentTime * 1000 - this.startTime : 0;
  };

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

  private schedule = async (song: string, startOffset: number) => {
    const songData = await (this.sources.length === 0 ? this.loadHtml5(`file://${song}`) : this.load(`file://${song}`));
    let startSeconds = startOffset > 0 ? startOffset : songData.startGap;
    let currentSwitchTime = this.switchTime;
    if (this.switchTime === 0) {
      currentSwitchTime = this.context.currentTime;
    }

    let nextSwitchTime = currentSwitchTime + songData.audioNode.duration() - startSeconds;
    if (this.sources.length === 0) {
      nextSwitchTime -= 0.1;
    }
    let start = currentSwitchTime === 0 ? this.context.currentTime : currentSwitchTime;
    await songData.audioNode.start(start, startSeconds);
    console.log('starting at', startSeconds);
    songData.audioNode.stop(nextSwitchTime);
    const source = {
      source: songData.audioNode,
      analyser: songData.analyser,
      gain: songData.gain,
      start,
      stop: nextSwitchTime,
      file: song,
    };
    if (this.sources.length === 0) {
      this.updateCurrent(source, startOffset);
    }
    this.sources.push(source);
    songData.audioNode.onEnded(async (current: AudioNodeWrapper) => {
      // don't fire when stopped because we don't want to play the next track (sources will be empty when stopped)
      // Sometimes this event fires twice so check the source to ensure we only call onFinished once
      if (!this.sources.length || current !== this.sources[0].source) {
        return;
      }
      // first source in the queue finished, don't need it anymore
      this.sources.shift();
      if (this.sources.length) {
        this.updateCurrent(this.sources[0], 0);
      }
      if (this.sources.length === 1) {
        // Last scheduled song, schedule the next batch
        await this.initialize(this.unscheduled, 0, false);
      } else if (this.unscheduled.length === 0) {
        // No more songs, clear all data
        this.stop();
      }
    });
    this.switchTime = nextSwitchTime;
  };

  private updateCurrent = (songData: ScheduledSource, startOffset: number) => {
    audioQueue.currentAnalyser = songData.analyser;
    audioQueue.currentGain = songData.gain;
    audioQueue.durationMillis.next(songData.source.duration() * 1000);
    this.startTime = (this.context.currentTime - startOffset) * 1000;
    this.playingSource.next(songData.file);
  };

  public stop = (seekTime: number = 0) => {
    // If we're seeking, reset the start time since we're rescheduling all sources
    // Otherwise reset the current source
    if (seekTime > 0) {
      this.startTime = (this.context.currentTime - seekTime) * 1000;
    } else {
      this.playingSource.next('');
    }
    for (let song of this.sources) {
      song.source.stopNow();
    }
    this.switchTime = 0;
    this.isPaused = false;
    this.setIsPlaying(seekTime > 0);
    this.sources = [];
    this.unscheduled = [];
    if (this.currentGain) {
      this.currentGain.gain.value = 0;
    }
    this.context.resume();
  };

  private setIsPlaying(isPlaying: boolean) {
    this.isPlaying = isPlaying;
    this.isPlayingEvent.next(isPlaying);
  }

  private scheduleAll = async (songQueue: string[], initialStartSeconds: number) => {
    if (songQueue.length) {
      const scheduleNow = songQueue.slice(0, Math.min(2, songQueue.length));
      if (songQueue.length > 2) {
        this.unscheduled = songQueue.filter((_, index) => index > 1);
      } else {
        this.unscheduled = [];
      }
      for (let song of scheduleNow) {
        console.log(song);
        await this.schedule(song, song === songQueue[0] ? initialStartSeconds : 0);
      }
    }
  };

  public start = async (songQueue: string[], startSeconds: number = 0) => {
    await this.initialize(songQueue, startSeconds, true);
  };

  private initialize = async (songQueue: string[], initialStartSeconds: number, stopBeforeStart: boolean) => {
    if (this.isPaused) {
      this.isPaused = false;
      this.setVolumeTemporary(this.volume);
      this.context.resume();
      // Starting the song that's currently paused, don't reschedule
      if (this.sources.length && songQueue[0] === this.sources[0].file) {
        this.setIsPlaying(true);
        return;
      } else {
        this.stop();
      }
    }
    if (this.isPlaying && stopBeforeStart) {
      this.stop(initialStartSeconds);
    }
    await this.scheduleAll(songQueue, initialStartSeconds);
    if (!this.isPlaying) {
      this.setIsPlaying(true);
    }
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
    this.setIsPlaying(false);
    this.isPaused = true;
  }

  public seek(millis: number) {
    if (this.sources.length) {
      this.sources[0].source.seek(millis);
    }
  }
}

export const audioQueue = new AudioQueue();
