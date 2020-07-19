import { round } from "lodash";
import _ from "lodash";

export const shadeColor = (color: string, percent: number) => {

    let [R, G, B] = hexToRgb(color);

    R = round(R * (100 + percent) / 100);
    G = round(G * (100 + percent) / 100);
    B = round(B * (100 + percent) / 100);

    R = (R<255)?R:255;  
    G = (G<255)?G:255;  
    B = (B<255)?B:255;  

    let RR = ((R.toString(16).length===1)?"0"+R.toString(16):R.toString(16));
    let GG = ((G.toString(16).length===1)?"0"+G.toString(16):G.toString(16));
    let BB = ((B.toString(16).length===1)?"0"+B.toString(16):B.toString(16));

    return "#"+RR+GG+BB;
}

export const hexToRgb = (hex: string) => {
    let result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    if (result === null) {
        return [];
    }
    return [parseInt(result[1], 16),parseInt(result[2], 16),parseInt(result[3], 16)];
}

export const hexToRgbStr = (hex: string) => _.join(hexToRgb(hex), ',');

// https://awik.io/determine-color-bright-dark-using-javascript/
export const isLight = (hex: string) => {
    const [r, g, b] = hexToRgb(hex);
    return Math.sqrt(0.299 * (r ** 2) + 0.114 * (b ** 2) + 0.587 * (g ** 2)) > 127.5;
}