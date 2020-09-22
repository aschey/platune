import { configureStore } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';

import songsReducer from './songs';
import tagsReducer from './tags';

const store = configureStore({
  reducer: {
    songs: songsReducer,
    tags: tagsReducer,
  },
});

export default store;

export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
