import { Theme } from "./theme";
import { darkTheme } from "./dark";
import { lightTheme } from "./light";
import { setCssVar, hexToRgb } from "../util";
import { shadeColor } from "./colorMixer";

export const themes: Record<string, Theme> = {
    'dark': darkTheme,
    'light': lightTheme
}

const intents = ['Primary', 'Success', 'Warning', 'Danger'];

const capitalize = (s: string) => {
    return s.charAt(0).toUpperCase() + s.slice(1)
}

export const applyTheme = (theme: string) => {
    let themeObj = themes[theme];
    for (let prop of Object.getOwnPropertyNames(themeObj)) {
        setCssVar(`--${prop}`, hexToRgb(themeObj[prop]));
    }

    for (let intent of intents) {
        setCssVar(`--hover${intent}`, hexToRgb(shadeColor(themeObj[`intent${intent}`], -20)));
        setCssVar(`--active${intent}`, hexToRgb(shadeColor(themeObj[`intent${intent}`], -25)));
    }
}