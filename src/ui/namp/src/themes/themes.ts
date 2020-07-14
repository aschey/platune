import { Theme } from "./theme";
import { darkTheme } from "./dark";
import { lightTheme } from "./light";
import { setCssVar, hexToRgb } from "../util";

export const themes: Record<string, Theme> = {
    'dark': darkTheme,
    'light': lightTheme
}

export const applyTheme = (theme: string) => {
    let themeObj = themes[theme];
    for (let prop of Object.getOwnPropertyNames(themeObj)) {
        setCssVar(`--${prop}`, hexToRgb(themeObj[prop]));
    }
}