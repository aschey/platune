import React from 'react';
import { Table, Column, TableRowProps, defaultTableRowRenderer, TableHeaderRowProps } from 'react-virtualized';
import { Song } from '../models/song';
import { FlexCol } from './FlexCol';
import { Icon, Tag, Label, Text, Intent, Button } from '@blueprintjs/core';
import { audioQueue } from '../audio';
import { useObservable } from 'rxjs-hooks';
import { defaultHeaderRowRenderer } from 'react-virtualized/dist/es/Table';
import { FlexRow } from './FlexRow';
import { random } from 'lodash';

interface QueueGridProps {
  queuedSongs: Song[];
}

export const QueueGrid: React.FC<QueueGridProps> = ({ queuedSongs }) => {
  const playingSource = useObservable(() => audioQueue.playingSource);
  const width = 190;

  const rowRenderer = (props: TableRowProps) => {
    props.style.width -= 11;
    props.style.boxShadow =
      queuedSongs[props.index].path === playingSource
        ? 'inset 0 0 2px 2px rgba(var(--intent-success), 0.3)'
        : 'inset 0 -1px 0 rgba(16, 22, 26, 0.3), inset -1px 0 0 rgba(16, 22, 26, 0.3)';
    props.onRowDoubleClick = params => {
      audioQueue.start(queuedSongs[params.index].path);
    };
    return defaultTableRowRenderer(props);
  };

  const headerRowRenderer = (props: TableHeaderRowProps) => {
    props.style.margin = 0;
    props.style.padding = 0;
    return defaultHeaderRowRenderer(props);
  };

  return (
    <div style={{ maxWidth: width, paddingLeft: 5, paddingRight: 5 }}>
      <div style={{ minHeight: 10, background: 'rgba(var(--background-secondary), 1)', minWidth: width + 5 }} />
      <FlexCol
        style={{
          fontSize: 16,
          background: 'rgba(var(--background-secondary), 1)',
          paddingBottom: 5,
          minWidth: width + 5,
        }}
      >
        <FlexRow>
          <div style={{ flex: 1 }} />
          <Text>Tags</Text>
          <Button minimal small style={{ marginLeft: 5, padding: 0 }}>
            <Icon iconSize={14} icon='add' style={{ paddingBottom: 1 }} />
          </Button>
        </FlexRow>
      </FlexCol>
      <Table
        width={width}
        height={(window.innerHeight - 180) / 2}
        rowHeight={30}
        headerHeight={25}
        disableHeader={true}
        rowCount={queuedSongs.length}
        rowGetter={({ index }) => queuedSongs[index]}
        style={{ background: 'rgba(var(--background-secondary), 1)' }}
      >
        <Column
          dataKey=''
          width={width}
          cellRenderer={({ rowIndex }) => (
            <div style={{ paddingLeft: 5 }}>
              <Tag intent={[Intent.PRIMARY, Intent.DANGER, Intent.SUCCESS, Intent.WARNING][Math.round(random(0, 4))]}>
                {
                  <FlexRow>
                    <Text ellipsize className='tag-text'>
                      {queuedSongs[rowIndex].name}
                    </Text>

                    <Button minimal small style={{ minHeight: 20, minWidth: 20 }}>
                      <Icon iconSize={12} icon='edit' style={{ paddingBottom: 1 }} />
                    </Button>
                  </FlexRow>
                }
              </Tag>
            </div>
          )}
        />
      </Table>
      <div style={{ minHeight: 10, background: 'rgba(var(--background-secondary), 1)', minWidth: width + 5 }} />
      <FlexCol
        style={{
          fontSize: 16,
          background: 'rgba(var(--background-secondary), 1)',
          paddingBottom: 5,
          minWidth: width + 5,
        }}
      >
        Now Playing
      </FlexCol>
      <Table
        width={width}
        height={(window.innerHeight - 180) / 2}
        rowHeight={70}
        disableHeader={true}
        headerHeight={25}
        headerRowRenderer={headerRowRenderer}
        rowCount={queuedSongs.length}
        rowGetter={({ index }) => queuedSongs[index]}
        rowRenderer={rowRenderer}
        style={{ background: 'rgba(var(--background-secondary), 1)' }}
      >
        <Column
          dataKey=''
          width={50}
          cellRenderer={({ rowIndex }) => (
            <div style={{ paddingLeft: 10, fontSize: 12 }}>
              {queuedSongs[rowIndex].path === playingSource ? (
                <Icon icon='volume-up' style={{ color: 'rgba(var(--intent-success), 1)' }} />
              ) : (
                <Text ellipsize>{rowIndex + 1}</Text>
              )}
            </div>
          )}
        />
        <Column
          width={width - 50}
          dataKey='name'
          cellRenderer={({ rowIndex }) => (
            <FlexCol center={false}>
              <Text ellipsize>{queuedSongs[rowIndex].name}</Text>
              <Text ellipsize className='secondary-text'>
                {queuedSongs[rowIndex].album}
              </Text>
              <Text ellipsize className='secondary-text'>
                {queuedSongs[rowIndex].artist}
              </Text>
            </FlexCol>
          )}
        />
      </Table>
    </div>
  );
};
