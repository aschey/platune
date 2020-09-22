import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { buildQueries } from '@testing-library/react';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';

export const fetchSongs = createAsyncThunk('songs', async (queryString?: string) => {
  const url = queryString ? `/songs?${queryString}` : '/songs';
  return getJson<Song[]>(url);
});

export type SliceState = { state: 'idle' | 'pending' | 'finished'; data: Song[] };

export interface SongState {
  songs: SliceState;
}

const initialState: SliceState = { state: 'idle', data: [] };

const songsSlice = createSlice({
  name: 'songs',
  initialState,
  reducers: {},
  extraReducers: builder => {
    builder.addCase(fetchSongs.pending, (state, action) => {
      state.state = 'pending';
    });
    builder.addCase(fetchSongs.fulfilled, (state, { payload }) => {
      state.state = 'finished';
      payload.forEach((song, i) => (song.index = i));
      state.data = payload;
      //console.log(state.data.entries().next());
    });
  },
});

export const selectSongs = (state: SongState) => {
  debugger;
  return state.songs.data;
};

export default songsSlice.reducer;
