import { darkTheme } from "./dark";

export interface Theme {
    textPrimary: string,
    textSecondary: string,
    backgroundPrimary: string,
    backgroundSecondary: string,
    success: string,
    primary: string,
    warning: string,
    danger: string
    [key: string]: string
}

