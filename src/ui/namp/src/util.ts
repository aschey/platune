import {Rgb} from './models/rgb';

export const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms)); 

export const range = (n: number) => Array.from({length: n}, (value, key) => key);

export const formatMs = (ms: number) => {
    if (ms < 0) {
        ms = 0;
    }
    const millisInSec = 1000;
    const secsInHr = 3600;
    const secsInMin = 60;
    let totalSecs = ms / millisInSec;
    let hrs = Math.floor(totalSecs / secsInHr);
    let mins = Math.floor((totalSecs % secsInHr) / secsInMin);
    let secs = Math.floor(totalSecs % secsInMin);
    return hrs > 0 ? [hrs, padNum(mins), padNum(secs)].join(':') : [mins, padNum(secs)].join(':');
}

const padNum = (num: number) => num.toString().padStart(2, '0');

export const setCssVar = (name: string, value: string) =>
    document.documentElement.style.setProperty(name, value);

export const formatRgb = (rgb: Rgb) => `${rgb.r},${rgb.g},${rgb.b}`;

export const capitalize = (word: string) => {
    if (!word) return word;
    return word[0].toUpperCase() + word.substr(1).toLowerCase();
}
