import { Theme } from './theme';
import { darkTheme } from './dark';
import { lightTheme } from './light';
import { setCssVar } from '../util';
import { shadeColor, hexToRgbStr } from './colorMixer';

export const themes: Record<string, Theme> = {
  dark: darkTheme,
  light: lightTheme,
};

const intents = ['Primary', 'Success', 'Warning', 'Danger'];
const addDefaults = ['gridSelectedBackground', 'gridStripe1', 'gridStripe2'];

const camelCaseToKebabCase = (str: string) => str.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
const camelCaseToCssVar = (str: string) => `--${camelCaseToKebabCase(str)}`;

export const applyTheme = (theme: string) => {
  let themeObj = themes[theme];
  const cssColorBlend = (prop: string, amount: number) => hexToRgbStr(shadeColor(themeObj[prop], amount));

  for (let prop of Object.getOwnPropertyNames(themeObj)) {
    setCssVar(camelCaseToCssVar(prop), hexToRgbStr(themeObj[prop]));
  }

  for (let defaultVar of addDefaults) {
    setCssVar(`${camelCaseToCssVar(defaultVar)}-default`, hexToRgbStr(themeObj[defaultVar]));
  }

  for (let intent of intents) {
    setCssVar(`--${intent.toLowerCase()}-hover`, cssColorBlend(`intent${intent}`, -20));
    setCssVar(`--${intent.toLowerCase()}-active`, cssColorBlend(`intent${intent}`, -25));
  }

  setCssVar('--card-shadow', cssColorBlend('backgroundSecondary', -20));
  setCssVar('--dialog-header', cssColorBlend('dialogBackground', 5));
  setCssVar('--cell-background', cssColorBlend('tableBackground', 10));
  setCssVar('--button-background-hover', cssColorBlend('buttonBackground', 10));
  setCssVar('--button-background-active', cssColorBlend('buttonBackground', 20));
  setCssVar('--grid-selected-shadow-1-default', hexToRgbStr(themeObj['gridSelectedShadow']));
  setCssVar('--grid-selected-shadow-2-default', cssColorBlend('gridSelectedShadow', 10));
  setCssVar('--visualizer-background', cssColorBlend('backgroundMain', -20));
};
