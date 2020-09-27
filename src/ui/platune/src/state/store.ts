import { configureStore, getDefaultMiddleware } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';
import logger from 'redux-logger';

import songsReducer from './songs';
import selectedGridReducer from './selectedGrid';
import searchReducer from './search';

const store = configureStore({
  reducer: {
    songs: songsReducer,
    selectedGrid: selectedGridReducer,
    search: searchReducer,
  },
  middleware: getDefaultMiddleware =>
    getDefaultMiddleware({ immutableCheck: false, serializableCheck: false }).concat(logger),
});

export default store;

export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
