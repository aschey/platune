import React from 'react';
import { Table, Column, TableRowProps, defaultTableRowRenderer } from 'react-virtualized';
import { Song } from '../models/song';
import { FlexCol } from './FlexCol';
import { Icon } from '@blueprintjs/core';
import { audioQueue } from '../audio';
import { useObservable } from 'rxjs-hooks';

interface QueueGridProps {
  queuedSongs: Song[];
}

export const QueueGrid: React.FC<QueueGridProps> = ({ queuedSongs }) => {
  const playingSource = useObservable(() => audioQueue.playingSource);
  const rowRenderer = (props: TableRowProps) => {
    props.style.boxShadow = 'inset 0 -1px 0 rgba(16, 22, 26, 0.3), inset -1px 0 0 rgba(16, 22, 26, 0.3)';
    props.onRowDoubleClick = params => {
      audioQueue.start(queuedSongs.filter((s, i) => i >= params.index).map(s => s.path));
    };
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
            {queuedSongs[rowIndex].path === playingSource ? <Icon icon='volume-up' /> : rowIndex + 1}
          </div>
        )}
      />
      <Column
        width={150}
        dataKey='name'
        cellRenderer={({ rowIndex }) => (
          <FlexCol>
            <div className='ellipsize'>{queuedSongs[rowIndex].name}</div>
            <div
              className='ellipsize'
              style={{
                fontSize: 12,
                color: 'rgba(var(--text-secondary), 0.8)',
              }}
            >
              {queuedSongs[rowIndex].album}
            </div>
            <div
              className='ellipsize'
              style={{
                fontSize: 12,
                color: 'rgba(var(--text-secondary), 0.8)',
              }}
            >
              {queuedSongs[rowIndex].artist}
            </div>
          </FlexCol>
        )}
      />
    </Table>
  );
};
