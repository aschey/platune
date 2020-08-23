import React from 'react';
import { Table, Column, TableRowProps, defaultTableRowRenderer } from 'react-virtualized';
import { Song } from '../models/song';
import { FlexCol } from './FlexCol';
import { Icon } from '@blueprintjs/core';

interface QueueGridProps {
  queuedSongs: Song[];
  queuePlayingRow: number;
}

export const QueueGrid: React.FC<QueueGridProps> = ({ queuedSongs, queuePlayingRow }) => {
  const rowRenderer = (props: TableRowProps) => {
    props.style.boxShadow = 'inset 0 -1px 0 rgba(16, 22, 26, 0.3), inset -1px 0 0 rgba(16, 22, 26, 0.3)';
    return defaultTableRowRenderer(props);
  };

  return (
    <Table
      width={200}
      height={500}
      rowHeight={60}
      headerHeight={25}
      rowCount={queuedSongs.length}
      disableHeader={true}
      rowGetter={({ index }) => queuedSongs[index]}
      rowRenderer={rowRenderer}
    >
      <Column
        dataKey=''
        width={50}
        cellRenderer={({ rowIndex }) => (
          <div style={{ paddingLeft: 5 }}>
            {rowIndex === queuePlayingRow ? <Icon icon='volume-up' /> : rowIndex + 1}
          </div>
        )}
      />
      <Column
        width={150}
        dataKey='name'
        cellRenderer={({ rowIndex }) => (
          <FlexCol>
            <div>{queuedSongs[rowIndex].name}</div>
            <div style={{ fontSize: 12, color: 'rgba(var(--text-secondary), 0.8)' }}>{queuedSongs[rowIndex].album}</div>
            <div style={{ fontSize: 12, color: 'rgba(var(--text-secondary), 0.8)' }}>
              {queuedSongs[rowIndex].artist}
            </div>
          </FlexCol>
        )}
      />
    </Table>
  );
};
