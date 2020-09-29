import React from 'react';
import { createContext, useContext, useEffect } from 'react';
import { useMemoizedState } from '../hooks/useMemoizedState';
import { isLight } from '../themes/colorMixer';
import { darkTheme } from '../themes/dark';
import { lightTheme } from '../themes/light';
import { applyTheme } from '../themes/themes';

const ThemeContext = createContext({
  theme: 'dark',
  themeVal: darkTheme,
  isLightTheme: false,
  setTheme: (_: string) => {},
});

export const useThemeContext = () => useContext(ThemeContext);

export const ThemeContextProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setTheme] = useMemoizedState('dark');
  const [themeVal, setThemeVal] = useMemoizedState(darkTheme);
  const [isLightTheme, setIsLightTheme] = useMemoizedState(false);

  useEffect(() => {
    applyTheme(theme);
    const newTheme = theme === 'light' ? lightTheme : darkTheme;
    setThemeVal(newTheme);
    setIsLightTheme(isLight(newTheme.backgroundMain));
  }, [theme]);

  return <ThemeContext.Provider value={{ theme, isLightTheme, setTheme, themeVal }}>{children}</ThemeContext.Provider>;
};
