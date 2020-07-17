import { darkTheme } from "./dark";

export interface Theme {
    intentPrimary: string,
    intentSuccess: string,
    intentWarning: string,
    intentDanger: string,
    textMain: string,
    textSecondary: string,
    backgroundMain: string,
    backgroundSecondary: string,
    textSuccess: string,
    textPrimary: string,
    textWarning: string,
    textDanger: string,
    textMuted: string,
    tableBackground: string,
    gridStripe1: string,
    gridStripe2: string,
    navbarBackground: string,
    dialogBackground: string,
    [key: string]: string
}

