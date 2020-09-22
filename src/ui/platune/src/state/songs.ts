import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { buildQueries } from '@testing-library/react';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';

type SongState = { state: 'idle' | 'pending' | 'finished'; data: Song[] };

const initialState: SongState = { state: 'idle', data: [] };

export const fetchSongs = createAsyncThunk('songs', async (queryString?: string) => {
  const url = queryString ? `/songs?${queryString}` : '/songs';
  return getJson<Song[]>(url);
});

const songsSlice = createSlice({
  name: 'songs',
  initialState,
  reducers: {},
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

export const selectSongs = (state: { songs: SongState }) => state.songs.data;

export default songsSlice.reducer;
