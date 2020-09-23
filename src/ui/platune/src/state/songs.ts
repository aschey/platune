import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { deleteJson, getJson, putJson } from '../fetchUtil';
import { EditSongTag } from '../models/editSongTag';
import { Song } from '../models/song';
import { SongTag } from '../models/songTag';
import { AppDispatch } from './store';

type SongState = {
  loadingState: 'idle' | 'pending' | 'finished';
  songData: Song[];
  tagData: SongTag[];
  filters: string;
};

const initialState: SongState = { loadingState: 'idle', songData: [], tagData: [], filters: '' };

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

export const fetchTags = createAsyncThunk('fetchTags', async () => getJson<SongTag[]>('/tags'));

export const addSongsToTag = createAsyncThunk<SongTag[], { tagId: number; songIds: number[] }, Thunk>(
  'addSongsToTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }, thunkApi) => {
    await putJson(`/tags/${tagId}/addSongs`, songIds);
    thunkApi.dispatch(songsSlice.actions.addTags({ tagId, songIds }));
    return getJson<SongTag[]>('/tags');
  }
);

export const removeSongsFromTag = createAsyncThunk<SongTag[], { tagId: number; songIds: number[] }, Thunk>(
  'removeSongsFromTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }, thunkApi) => {
    await putJson(`/tags/${tagId}/removeSongs`, songIds);
    thunkApi.dispatch(songsSlice.actions.removeTags({ tagId, songIds }));
    return getJson<SongTag[]>('/tags');
  }
);

export const addEditTag = createAsyncThunk('addEditTag', async (tag: EditSongTag) => {
  if (tag.id === null) {
    await putJson('/tags', tag);
  } else {
    await putJson(`/tags/${tag.id}`, tag);
  }
  return getJson<SongTag[]>('/tags');
});

export const deleteTag = createAsyncThunk('deleteTag', async (tagId: number) => {
  await deleteJson(`/tags/${tagId}`);
  return getJson<SongTag[]>('/tags');
});

const songsSlice = createSlice({
  name: 'songs',
  initialState,
  reducers: {
    setFilters: (state, { payload }: PayloadAction<string>) => {
      state.filters = payload;
    },
    addTags: (state, { payload }: PayloadAction<{ tagId: number; songIds: number[] }>) => {
      const tag = state.tagData.find(t => t.id === payload.tagId);
      if (!tag) {
        return;
      }
      let songCount = payload.songIds.length;
      for (let i = 0; i < state.songData.length && songCount > 0; i++) {
        const song = state.songData[i];
        if (payload.songIds.includes(song.id) && !song.tags.map(t => t.id).includes(payload.tagId)) {
          song.tags.push(tag);
          songCount--;
        }
      }
    },
    removeTags: (state, { payload }: PayloadAction<{ tagId: number; songIds: number[] }>) => {
      const tag = state.tagData.find(t => t.id === payload.tagId);
      if (!tag) {
        return;
      }
      let songCount = payload.songIds.length;
      for (let i = 0; i < state.songData.length && songCount > 0; i++) {
        const song = state.songData[i];
        const tagIds = song.tags.map(t => t.id);
        if (payload.songIds.includes(song.id) && song.tags.map(t => t.id).includes(payload.tagId)) {
          const index = tagIds.indexOf(payload.tagId);
          song.tags.splice(index, 1);
          songCount--;
        }
      }
    },
  },
  extraReducers: builder => {
    builder.addCase(fetchSongs.pending, state => {
      state.loadingState = 'pending';
    });
    builder.addCase(fetchSongs.fulfilled, (state, { payload }) => {
      state.loadingState = 'finished';
      payload.forEach((song, i) => (song.index = i));
      state.songData = payload;
    });
    builder.addCase(fetchTags.fulfilled, (state, { payload }) => {
      state.tagData = payload;
    });
    builder.addCase(addEditTag.fulfilled, (state, { payload }) => {
      state.tagData = payload;
    });
    builder.addCase(deleteTag.fulfilled, (state, { payload }) => {
      state.tagData = payload;
    });
    builder.addCase(addSongsToTag.fulfilled, (state, { payload }) => {
      state.tagData = payload;
    });
    builder.addCase(removeSongsFromTag.fulfilled, (state, { payload }) => {
      state.tagData = payload;
    });
  },
});

export const { setFilters } = songsSlice.actions;

export const selectSongs = (state: State) => state.songs.songData;

export const selectTags = (state: State) => state.songs.tagData;

export default songsSlice.reducer;
