import { NumericDictionary } from 'lodash';

export interface SongRequest {
  albumArtistId?: number;
  albumId?: number;
  artistId?: number;
  songName?: string;
  tagIds?: number[];
}
