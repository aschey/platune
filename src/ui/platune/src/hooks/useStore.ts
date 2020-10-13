import _ from 'lodash';
import { useCallback, useEffect } from 'react';
import create from 'zustand';
import { devtools } from 'zustand/middleware';
import shallow from 'zustand/shallow';
import { postJson } from '../fetchUtil';
import { FilterRequest } from '../models/filterRequest';
import { Song } from '../models/song';

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
    set({ filters });
  },
  setFilterTag: ({ tagId, append, toggle }: { tagId: number; append: boolean; toggle: boolean }) => {
    set(state => {
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
  },
}));

export const useFilters = () => {
  return useStore(
    useCallback((state: State) => ({ filters: state.filters, setFilters: state.setFilters }), []),
    shallow
  );
};

export const useTagFilters = () => {
  return useStore(
    useCallback((state: State) => ({ tagFilters: state.tagFilters, setFilterTag: state.setFilterTag }), []),
    shallow
  );
};
