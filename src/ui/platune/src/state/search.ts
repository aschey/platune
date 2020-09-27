import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';
import _ from 'lodash';
import { getJson } from '../fetchUtil';
import { Search } from '../models/search';
import { SearchRequest } from '../models/searchRequest';

const debounceSearch = _.debounce(async ({ searchString, limit, includeTags }: SearchRequest) => {
  let searchResult = await getJson<Search[]>(
    `/search?limit=${limit}&includeTags=${includeTags}&searchString=${encodeURIComponent(
      searchString
        .split(/\s+/)
        .map(s => `"${s}"`)
        .join(' ')
    )}*`
  );
  return searchResult;
});
export const fetchSearchResults = createAsyncThunk<Search[], SearchRequest>('search', async searchRequest => {
  let debounced = debounceSearch(searchRequest);
  if (debounced) {
    return debounced;
  }
  return [];
});

type SearchState = { searchResults: Search[] };
const initialState: SearchState = { searchResults: [] };

interface State {
  search: SearchState;
}

const searchSlice = createSlice({
  name: 'search',
  initialState,
  reducers: {
    clearSearch: state => {
      state.searchResults = [];
    },
  },
  extraReducers: builder => {
    builder.addCase(fetchSearchResults.fulfilled, (state, { payload }) => {
      state.searchResults = payload;
    });
  },
});

export const selectSearchResults = (state: State) => state.search.searchResults;

export const { clearSearch } = searchSlice.actions;

export default searchSlice.reducer;
