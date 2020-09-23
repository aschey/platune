import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { getJson } from '../fetchUtil';
import { Song } from '../models/song';
import { AppDispatch } from './store';

type SongState = { loadingState: 'idle' | 'pending' | 'finished'; data: Song[]; filters: string };

const initialState: SongState = { loadingState: 'idle', data: [], filters: '' };

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
    setFilters: (state, { payload }: PayloadAction<string>) => {
      state.filters = payload;
    },
  },
  extraReducers: builder => {
    builder.addCase(fetchSongs.pending, state => {
      state.loadingState = 'pending';
    });
    builder.addCase(fetchSongs.fulfilled, (state, { payload }) => {
      state.loadingState = 'finished';
      payload.forEach((song, i) => (song.index = i));
      state.data = payload;
    });
  },
});

export const { setFilters } = songsSlice.actions;

export const selectSongs = (state: State) => state.songs.data;

export default songsSlice.reducer;
