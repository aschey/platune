import useSWR from 'swr';
import { getJson, putJson } from '../fetchUtil';
import { SongTag } from '../models/songTag';
import { useSongs } from './useSongs';

const getTags = (url: string) => getJson<SongTag[]>(url);

const addSongsToTag = (tagId: number, songIds: number[]) => putJson(`/tags/${tagId}/addSongs`, songIds);

const removeSongsFromTag = (tagId: number, songIds: number[]) => putJson(`/tags/${tagId}/removeSongs`, songIds);

export const useTags = () => useSWR('/tags', getTags, { revalidateOnFocus: false });

export const useAddSongsToTag = () => {
  const { data: tags, mutate: mutateTags } = useTags();
  const { data: songData, mutate: songMutate } = useSongs();
  const songs = songData?.slice() ?? [];
  return async (tagId: number, songIds: number[]) => {
    await addSongsToTag(tagId, songIds);
    mutateTags();

    const tag = tags?.find(t => t.id === tagId);
    if (!tag) {
      return;
    }
    let songCount = songIds.length;
    for (let i = 0; i < songs.length && songCount > 0; i++) {
      const song = songs[i];
      if (songIds.includes(song.id) && !song.tags.map(t => t.id).includes(tagId)) {
        song.tags.push(tag);
        songCount--;
      }
    }
    songMutate(songs);
  };
};

export const useRemoveSongsFromTag = () => {
  const { data: tags, mutate: mutateTags } = useTags();
  const { data: songData, mutate: songMutate } = useSongs();
  const songs = songData?.slice() ?? [];
  return async (tagId: number, songIds: number[]) => {
    await removeSongsFromTag(tagId, songIds);
    mutateTags();
    const tag = tags?.find(t => t.id === tagId);
    if (!tag) {
      return;
    }
    let songCount = songIds.length;
    for (let i = 0; i < songs.length && songCount > 0; i++) {
      const song = songs[i];
      const tagIds = song.tags.map(t => t.id);
      if (songIds.includes(song.id) && song.tags.map(t => t.id).includes(tagId)) {
        const index = tagIds.indexOf(tagId);
        song.tags.splice(index, 1);
        songCount--;
      }
    }
    songMutate(songs);
  };
};
