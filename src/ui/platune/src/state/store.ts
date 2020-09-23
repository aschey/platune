import { configureStore } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';

import songsReducer from './songs';
import tagsReducer from './tags';
import selectedGridReducer from './selectedGrid';

const store = configureStore({
  reducer: {
    songs: songsReducer,
    tags: tagsReducer,
    selectedGrid: selectedGridReducer,
  },
});

export default store;

export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
