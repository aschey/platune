import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { buildQueries } from '@testing-library/react';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { AppDispatch } from './store';

type SongState = { state: 'idle' | 'pending' | 'finished'; data: Song[]; filters: string };

const initialState: SongState = { state: 'idle', data: [], filters: '' };

interface State {
  songs: SongState;
}

interface Thunk {
  dispatch: AppDispatch;
  state: State;
}

export const fetchSongs = createAsyncThunk<Song[], undefined, Thunk>('songs', async (_, thunkApi) => {
  const state = thunkApi.getState();
  const url = state.songs.filters.length ? `/songs?${state.songs.filters}` : '/songs';
  return getJson<Song[]>(url);
});

const songsSlice = createSlice({
  name: 'songs',
  initialState,
  reducers: {
    setFilters: (state, { payload }) => {
      state.filters = payload;
    },
  },
  extraReducers: builder => {
    builder.addCase(fetchSongs.pending, state => {
      state.state = 'pending';
    });
    builder.addCase(fetchSongs.fulfilled, (state, { payload }) => {
      state.state = 'finished';
      payload.forEach((song, i) => (song.index = i));
      state.data = payload;
    });
  },
});

export const { setFilters } = songsSlice.actions;

export const selectSongs = (state: { songs: SongState }) => state.songs.data;

export default songsSlice.reducer;
