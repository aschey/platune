import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import { deleteJson, getJson, postJson, putJson } from '../fetchUtil';
import { EditSongTag } from '../models/editSongTag';
import { SongTag } from '../models/songTag';

type TagState = { state: 'idle' | 'pending' | 'finished'; data: SongTag[] };

export const fetchTags = createAsyncThunk('fetchTags', async () => getJson<SongTag[]>('/tags'));

export const addSongsToTag = createAsyncThunk(
  'addSongsToTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }) => {
    await putJson(`/tags/${tagId}/addSongs`, songIds);
    return getJson<SongTag[]>('/tags');
  }
);

export const removeSongsFromTag = createAsyncThunk(
  'removeSongsFromTag',
  async ({ tagId, songIds }: { tagId: number; songIds: number[] }) => {
    await putJson(`/tags/${tagId}/removeSongs`, songIds);
    return getJson<SongTag[]>('/tags');
  }
);

export const addEditTag = createAsyncThunk('addEditTag', async (tag: EditSongTag) => {
  if (tag.id === null) {
    await postJson('/tags', tag);
  } else {
    await putJson(`/tags/${tag.id}`, tag);
  }
  return getJson<SongTag[]>('/tags');
});

export const deleteTag = createAsyncThunk('deleteTag', async (tagId: number) => {
  await deleteJson(`/tags/${tagId}`);
  return getJson<SongTag[]>('/tags');
});

const tagsSlice = createSlice({
  name: 'tags',
  initialState: { state: 'idle', data: [] } as TagState,
  reducers: {},
  extraReducers: builder => {
    builder.addCase(fetchTags.fulfilled, (state, { payload }) => {
      state.data = payload;
    });
    builder.addCase(addEditTag.fulfilled, (state, { payload }) => {
      state.data = payload;
    });
    builder.addCase(deleteTag.fulfilled, (state, { payload }) => {
      state.data = payload;
    });
    builder.addCase(addSongsToTag.fulfilled, (state, { payload }) => {
      state.data = payload;
    });
    builder.addCase(removeSongsFromTag.fulfilled, (state, { payload }) => {
      state.data = payload;
    });
  },
});

export const selectTags = (state: { tags: TagState }) => state.tags.data;

export default tagsSlice.reducer;
