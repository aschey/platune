import { useSelector } from 'react-redux';
import useSWR from 'swr';
import { postJson } from '../fetchUtil';
import { FilterRequest } from '../models/filterRequest';
import { Song } from '../models/song';
import { useFilters, useTagFilters } from './useStore';

const getSongs = async (url: string, filters: FilterRequest, tagFilters: number[]) => {
  let res = await postJson<Song[]>(url, { ...filters, tagIds: tagFilters });
  res.forEach((s, i) => (s.index = i));
  return res;
};

export const useSongs = () => {
  const { filters } = useFilters();
  const { tagFilters } = useTagFilters();
  return useSWR(['/songs', filters, tagFilters], getSongs, { revalidateOnFocus: false });
};
