import { configureStore, getDefaultMiddleware } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';
import logger from 'redux-logger';

import songsReducer from './songs';
import tagsReducer from './tags';
import selectedGridReducer from './selectedGrid';

const store = configureStore({
  reducer: {
    songs: songsReducer,
    tags: tagsReducer,
    selectedGrid: selectedGridReducer,
  },
  middleware: getDefaultMiddleware => getDefaultMiddleware().concat(logger),
});

export default store;

export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
