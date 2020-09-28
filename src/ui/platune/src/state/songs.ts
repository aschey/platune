import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { deleteJson, getJson, postJson, putJson } from '../fetchUtil';
import { EditSongTag } from '../models/editSongTag';
import { Song } from '../models/song';
import { FilterRequest } from '../models/filterRequest';
import { SongTag } from '../models/songTag';
import { AppDispatch } from './store';

type SongState = {
  loadingState: 'idle' | 'pending' | 'finished';
  songData: Song[];
  tagData: SongTag[];
  filters: FilterRequest;
  tagFilters: number[];
};

const initialState: SongState = {
  loadingState: 'idle',
  songData: [],
  tagData: [],
  filters: {},
  tagFilters: [],
};

interface State {
  songs: SongState;
}

interface Thunk {
  dispatch: AppDispatch;
  state: State;
  getState: () => State;
}

const getSongs = (thunkApi: { getState: () => State }) => {
  const state = thunkApi.getState();
  return postJson<Song[]>('/songs', { ...state.songs.filters, tagIds: state.songs.tagFilters });
};

const getTags = () => getJson<SongTag[]>('/tags');

export const fetchSongs = createAsyncThunk<Song[], undefined, Thunk>('songs', async (_, thunkApi) => {
  return getSongs(thunkApi);
});

export const fetchTags = createAsyncThunk('fetchTags', async () => getJson<SongTag[]>('/tags'));

export const addSongsToTag = createAsyncThunk<SongTag[], { tagId: number; songIds: number[] }, Thunk>(
  'addSongsToTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }, thunkApi) => {
    await putJson(`/tags/${tagId}/addSongs`, songIds);
    thunkApi.dispatch(songsSlice.actions.addTags({ tagId, songIds }));
    return getTags();
  }
);

export const removeSongsFromTag = createAsyncThunk<SongTag[], { tagId: number; songIds: number[] }, Thunk>(
  'removeSongsFromTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }, thunkApi) => {
    await putJson(`/tags/${tagId}/removeSongs`, songIds);
    thunkApi.dispatch(songsSlice.actions.removeTags({ tagId, songIds }));
    return getTags();
  }
);

export const addEditTag = createAsyncThunk('addEditTag', async (tag: EditSongTag) => {
  if (tag.id === undefined) {
    await postJson('/tags', tag);
  } else {
    await putJson(`/tags/${tag.id}`, tag);
  }
  return getTags();
});

export const deleteTag = createAsyncThunk('deleteTag', async (tagId: number) => {
  await deleteJson(`/tags/${tagId}`);
  return getTags();
});

export const setFilterTag = createAsyncThunk<Song[], { tagId: number; append: boolean; toggle: boolean }, Thunk>(
  'setFilterTag',
  async ({ tagId, append, toggle }, thunkApi) => {
    thunkApi.dispatch(songsSlice.actions.setFilterTag({ tagId, append, toggle }));
    return getSongs(thunkApi);
  }
);

export const setFilters = createAsyncThunk<Song[], FilterRequest, Thunk>('setFilters', async (request, thunkApi) => {
  thunkApi.dispatch(songsSlice.actions.setFilters(request));
  return getSongs(thunkApi);
});

const songsSlice = createSlice({
  name: 'songs',
  initialState,
  reducers: {
    setFilters: (state, { payload }: PayloadAction<FilterRequest>) => {
      state.filters = payload;
    },
    setFilterTag: (state, { payload }: PayloadAction<{ tagId: number; append: boolean; toggle: boolean }>) => {
      const { tagId, append, toggle } = payload;
      const tagIds = state.tagFilters;
      if (tagIds?.includes(tagId) && toggle) {
        tagIds.splice(tagIds.indexOf(tagId), 1);
      } else if (tagIds === undefined || !append) {
        state.tagFilters = [tagId];
      } else if (!tagIds.includes(tagId)) {
        tagIds.push(tagId);
      }
    },
    removeFilterTag: (state, { payload }: PayloadAction<number>) => {
      if (state.tagFilters === undefined) {
        state.tagFilters = [payload];
      } else {
        state.tagFilters.push(payload);
      }
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
    const setTagData = (state: SongState, { payload }: PayloadAction<SongTag[]>) => {
      state.tagData = payload;
    };
    const setSongData = (state: SongState, { payload }: PayloadAction<Song[]>) => {
      state.loadingState = 'finished';
      payload.forEach((song, i) => (song.index = i));
      state.songData = payload;
    };
    builder.addCase(fetchSongs.pending, state => {
      state.loadingState = 'pending';
    });
    builder
      .addCase(fetchSongs.fulfilled, setSongData)
      .addCase(setFilterTag.fulfilled, setSongData)
      .addCase(setFilters.fulfilled, setSongData)
      .addCase(fetchTags.fulfilled, setTagData)
      .addCase(addEditTag.fulfilled, setTagData)
      .addCase(deleteTag.fulfilled, setTagData)
      .addCase(addSongsToTag.fulfilled, setTagData)
      .addCase(removeSongsFromTag.fulfilled, setTagData);
  },
});

export const selectSongs = (state: State) => state.songs.songData;

export const selectTags = (state: State) => state.songs.tagData;

export const selectChosenTags = (state: State) => state.songs.tagFilters;

export default songsSlice.reducer;
