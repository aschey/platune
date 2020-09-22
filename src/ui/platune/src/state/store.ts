import { configureStore } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';

import songsReducer from './songs';

const store = configureStore({
  reducer: {
    songs: songsReducer,
  },
});

export default store;

export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
