export const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms)); 
export const range = (n: number) => Array.from({length: n}, (value, key) => key);