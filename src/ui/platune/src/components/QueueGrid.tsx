import React from 'react';
import { Table, Column } from 'react-virtualized';
import { Song } from '../models/song';

export const QueueGrid: React.FC<{ queuedSongs: Song[] }> = ({ queuedSongs }) => {
  return (
    <Table
      width={200}
      height={500}
      rowHeight={25}
      headerHeight={25}
      rowCount={queuedSongs.length}
      rowGetter={({ index }) => queuedSongs[index]}
    >
      <Column
        headerRenderer={props => 'Title'}
        width={150}
        dataKey='name'
        cellRenderer={({ rowIndex }) => queuedSongs[rowIndex].name}
      />
    </Table>
  );
};
