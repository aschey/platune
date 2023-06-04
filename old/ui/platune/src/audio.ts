import { BehaviorSubject, interval, Observable } from 'rxjs';
import { map } from 'rxjs/operators';
import { sleep } from './util';

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

export enum PlaybackState {
  Playing,
  Paused,
  Stopped,
}

class AudioNodeWrapper {
  private bufferNode: AudioBufferSourceNode | null;
  private htmlNode: HTMLAudioElement | null;

  constructor(node: AudioBufferSourceNode | HTMLAudioElement) {
    if (node instanceof HTMLAudioElement) {
      this.htmlNode = node;
      this.bufferNode = null;
    } else {
      this.bufferNode = node;
      this.htmlNode = null;
    }
  }

  public duration = () => (this.htmlNode ? this.htmlNode?.duration : this.bufferNode?.buffer?.duration) ?? 0;

  public start = async (when: number, offset: number) => {
    if (this.htmlNode) {
      this.htmlNode.currentTime = offset;
      await this.htmlNode?.play();
    } else {
      this.bufferNode?.start(when, offset);
    }
  };

  public stop = (when: number) => {
    if (this.bufferNode) {
      this.bufferNode?.stop(when);
    }
  };

  public stopNow = () => {
    if (this.htmlNode) {
      this.htmlNode.pause();
      this.htmlNode.currentTime = 0;
      this.htmlNode.src = '';
    } else {
      this.bufferNode?.stop();
    }
  };

  public onEnded = (handler: (current: AudioNodeWrapper) => Promise<void>) => {
    if (this.htmlNode) {
      this.htmlNode.onended = async () => await handler(this);
    } else {
      this.bufferNode?.addEventListener('ended', () => handler(this));
    }
  };

  public disconnect = () => {
    if (this.bufferNode) {
      this.bufferNode.disconnect();
      this.bufferNode = null;
    }
  };

  public isHtml = () => this.htmlNode !== null;
}

class AudioQueue {
  switchTime: number;
  finishCounter: number;
  context: AudioContext;
  sources: ScheduledSource[];
  playbackState: BehaviorSubject<PlaybackState>;
  playingSource: BehaviorSubject<string>;
  durationMillis: BehaviorSubject<number>;
  progress: Observable<number>;
  currentAnalyser: AnalyserNode | null;
  volume: number;
  startTime: number;
  queuedSongs: BehaviorSubject<string[]>;
  private currentGain: GainNode | null;
  private htmlAudio: HtmlAudioMetadata;
  private requestsToIgnore: number[];
  private pendingSchedules: boolean[];

  constructor() {
    this.switchTime = 0;
    this.finishCounter = 0;
    this.context = new AudioContext();
    this.sources = [];
    this.playbackState = new BehaviorSubject<PlaybackState>(PlaybackState.Stopped);
    this.currentAnalyser = null;
    this.currentGain = null;
    this.volume = 1.0;
    this.durationMillis = new BehaviorSubject<number>(0);
    this.playingSource = new BehaviorSubject<string>('');
    this.queuedSongs = new BehaviorSubject<string[]>([]);
    this.startTime = 0;
    this.progress = interval(200).pipe(map(this.getCurrentTime));
    this.requestsToIgnore = [];
    this.pendingSchedules = [];

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
    return this.playbackState.value !== PlaybackState.Stopped ? this.context.currentTime * 1000 - this.startTime : 0;
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
          audioNode: new AudioNodeWrapper(element),
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
    let arrayBuffer: ArrayBuffer | null = await data.arrayBuffer();
    // Todo: cache audio buffers if they are large enough to warrant caching
    const audioBuffer = await this.context.decodeAudioData(arrayBuffer);
    arrayBuffer = null;
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
      audioNode: new AudioNodeWrapper(source),
      analyser,
      gain,
      startGap,
      endGap,
    };
  };

  private shouldSchedule = (start: number) => {
    const priorReqs = this.requestsToIgnore.filter(d => d > start).sort();
    if (priorReqs.length) {
      this.requestsToIgnore.splice(this.requestsToIgnore.indexOf(priorReqs[0]), 1);
      return false;
    }
    return true;
  };

  private schedule = async (songIndex: number, startOffset: number, startTime: number) => {
    const song = this.queuedSongs.value[songIndex];
    if (!this.shouldSchedule(startTime)) {
      return false;
    }
    const songData = await (this.sources.length === 0 ? this.loadHtml5(`file://${song}`) : this.load(`file://${song}`));
    if (!this.shouldSchedule(startTime)) {
      return false;
    }
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
    console.log(`starting ${song} at`, startSeconds);
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
      if (!this.sources.length || current !== this.sources[0].source || this.isStopped()) {
        return;
      }
      // first source in the queue finished, don't need it anymore
      this.sources.shift();
      if (this.sources.length) {
        this.updateCurrent(this.sources[0], 0);
      }

      if (songIndex === this.queuedSongs.value.length - 1) {
        // No more songs, clear all data
        this.stop();
      } else if (this.sources.length === 1) {
        // Last scheduled song, schedule the next batch
        await this.initialize(this.queuedSongs.value[songIndex + 2], 0, false);
      }
    });
    this.switchTime = nextSwitchTime;

    return true;
  };

  private updateCurrent = (songData: ScheduledSource, startOffset: number) => {
    audioQueue.currentAnalyser = songData.analyser;
    audioQueue.currentGain = songData.gain;
    audioQueue.durationMillis.next(songData.source.duration() * 1000);
    this.startTime = (this.context.currentTime - startOffset) * 1000;
    this.playingSource.next(songData.file);
  };

  private clear = () => {
    // Clear out audio data to prevent memory leaks
    while (this.sources.length) {
      let song = this.sources.pop();
      if (!song) {
        continue;
      }
      song.source.stopNow();
      // Clear out song data to prevent memory leak
      // Html nodes have a shared source so we don't need to clear it
      if (!song.source.isHtml()) {
        song.source.disconnect();
        song.gain.disconnect();
        song.analyser.disconnect();
      }

      song = undefined;
    }
  };

  public stop = (seekTime: number = 0, willRestart: boolean = false) => {
    // If we're seeking, reset the start time since we're rescheduling all sources
    // Otherwise reset the current source
    this.playbackState.next(willRestart ? PlaybackState.Playing : PlaybackState.Stopped);
    if (this.pendingSchedules.length) {
      const now = new Date().getTime();
      this.requestsToIgnore.push(now);
    }

    if (seekTime > 0) {
      this.startTime = (this.context.currentTime - seekTime) * 1000;
    }
    if (!willRestart) {
      this.playingSource.next('');
    }

    this.clear();
    this.switchTime = 0;

    if (this.currentGain) {
      this.currentGain.gain.value = 0;
    }
    this.context.resume();
  };

  private scheduleAll = async (song: string, initialStartSeconds: number, startTime: number) => {
    const songIndex = this.queuedSongs.value.indexOf(song);

    this.pendingSchedules.push(true);
    const shouldContinue = await this.schedule(songIndex, initialStartSeconds, startTime);
    if (shouldContinue && songIndex < this.queuedSongs.value.length - 1) {
      await this.schedule(songIndex + 1, 0, startTime);
    }
    this.pendingSchedules.pop();
  };

  public start = async (song: string, startSeconds: number = 0) => {
    await this.initialize(song, startSeconds, true);
  };

  private initialize = async (song: string, initialStartSeconds: number, stopBeforeStart: boolean) => {
    const now = new Date().getTime();
    if (this.playbackState.value === PlaybackState.Paused) {
      this.setVolumeTemporary(this.volume);
      this.context.resume();

      // Starting the song that's currently paused, don't reschedule
      if (this.sources.length && song === this.sources[0].file) {
        this.playbackState.next(PlaybackState.Playing);
        return;
      } else {
        this.stop(0, true);
      }
    }
    if (this.isPlaying() && stopBeforeStart) {
      this.stop(initialStartSeconds, true);
    }
    if (!this.isPlaying()) {
      this.playbackState.next(PlaybackState.Playing);
    }
    await this.scheduleAll(song, initialStartSeconds, now);
  };

  private setVolumeTemporary = (volume: number) => {
    if (this.currentGain) {
      this.currentGain.gain.value = volume;
    }
    this.sources.forEach(s => (s.gain.gain.value = volume));
  };

  private currentIndex = () => {
    if (!this.sources.length) {
      return -1;
    }
    return this.queuedSongs.value.indexOf(this.sources[0].file);
  };

  public isPlaying = () => this.playbackState.value === PlaybackState.Playing;

  public isPaused = () => this.playbackState.value === PlaybackState.Paused;

  public isStopped = () => this.playbackState.value === PlaybackState.Stopped;

  public setVolume = (volume: number) => {
    this.setVolumeTemporary(volume);
    this.volume = volume;
  };

  public setQueue = (songs: string[]) => {
    this.queuedSongs.next(songs);
  };

  public pause = async () => {
    this.setVolumeTemporary(0);
    // Seems like there's a slight delay before the volume change takes so we have to wait for it to complete
    await sleep(100);
    await this.context.suspend();
    this.playbackState.next(PlaybackState.Paused);
  };

  public seek = (millis: number) => {
    if (this.sources.length) {
      const startSeconds = millis / 1000;
      audioQueue.start(audioQueue.sources[0].file, startSeconds);
    }
  };

  public next = async () => {
    const currentIndex = this.currentIndex();
    if (currentIndex === this.queuedSongs.value.length - 1) {
      return;
    }
    await this.initialize(this.queuedSongs.value[currentIndex + 1], 0, true);
  };

  public previous = async () => {
    const currentIndex = this.currentIndex();
    if (currentIndex < 1) {
      return;
    }

    await this.initialize(this.queuedSongs.value[currentIndex - 1], 0, true);
  };
}

export const audioQueue = new AudioQueue();
