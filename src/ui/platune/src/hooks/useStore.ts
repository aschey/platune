import _ from 'lodash';
import { useCallback, useEffect } from 'react';
import { QueryCache, useQueryCache } from 'react-query';
import create from 'zustand';
import { devtools } from 'zustand/middleware';
import shallow from 'zustand/shallow';
import { postJson } from '../fetchUtil';
import { FilterRequest } from '../models/filterRequest';
import { Song } from '../models/song';
import { getSongs } from './useSongs';

type State = {
  filters: FilterRequest;
  tagFilters: number[];
  setFilters: (filters: FilterRequest) => void;
  setFilterTag: ({ tagId, append, toggle }: { tagId: number; append: boolean; toggle: boolean }) => void;
};

const useStore = create<State>(set => ({
  filters: {},
  tagFilters: [],
  setFilters: (filters: FilterRequest) => {
    console.log(filters);
    set({ filters });
    //setTimeout(() => cache.invalidateQueries('songs'), 1);
  },
  setFilterTag: ({ tagId, append, toggle }: { tagId: number; append: boolean; toggle: boolean }) => {
    set(state => {
      console.log(tagId, append, toggle);
      let tagFilters = state.tagFilters.slice();
      if (tagFilters?.includes(tagId) && toggle) {
        tagFilters.splice(tagFilters.indexOf(tagId), 1);
      } else if (tagFilters === undefined || !append) {
        tagFilters = [tagId];
      } else if (!tagFilters.includes(tagId)) {
        tagFilters.push(tagId);
      }
      console.log(tagFilters);
      return { tagFilters };
    });
    //setTimeout(() => cache.invalidateQueries('songs'), 1);
  },
}));

const tagFilterSelector = (state: State) => state.tagFilters;
const filterSelector = (state: State) => state.filters;
const useFilterSelector = (state: State) => {
  const { filters, setFilters } = state;
  return { filters, setFilters };
};
const useTagFilterSelector = (state: State) => {
  const { tagFilters, setFilterTag } = state;
  return { tagFilters, setFilterTag };
};
export const useFilters = () => {
  const cache = useQueryCache();
  const tagFilters = useStore(tagFilterSelector);
  const subscription = useCallback(
    state => {
      cache.fetchQuery(['songs', state, tagFilters], getSongs);
    },
    [tagFilters, getSongs]
  );
  useStore.subscribe<FilterRequest>(subscription, filterSelector);
  return useStore(useFilterSelector, shallow);
};

export const useTagFilters = () => {
  const cache = useQueryCache();
  const filters = useStore(filterSelector);
  const subscription = useCallback(
    state => {
      console.log(state);
      cache.fetchQuery(['songs', filters, state], getSongs);
    },
    [filters, getSongs]
  );
  useStore.subscribe<number[]>(subscription, tagFilterSelector);
  return useStore(useTagFilterSelector, shallow);
};
