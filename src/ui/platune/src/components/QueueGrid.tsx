import { Button, Icon, Intent, Menu, MenuItem, Popover, Tag, Text } from '@blueprintjs/core';
import React, { useState, useRef, useEffect } from 'react';
import {
  Column,
  defaultTableRowRenderer,
  Table,
  TableHeaderRowProps,
  TableRowProps,
  List,
  ListRowProps,
} from 'react-virtualized';
import { defaultHeaderRowRenderer } from 'react-virtualized/dist/es/Table';
import { useObservable } from 'rxjs-hooks';
import { audioQueue } from '../audio';
import { Song } from '../models/song';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { AddEditTag } from './AddEditTag';
import {
  DraggableProvided,
  DraggableStateSnapshot,
  DraggableRubric,
  Droppable,
  DroppableProvided,
  DroppableStateSnapshot,
  Draggable,
} from 'react-beautiful-dnd';
import ReactDOM from 'react-dom';
import { deleteJson, getJson } from '../fetchUtil';
import { SongTag } from '../models/songTag';
import { hexToRgb } from '../themes/colorMixer';
import { theme } from './App';
import { toastSuccess } from '../appToaster';

interface QueueGridProps {
  queuedSongs: Song[];
}

export const QueueGrid: React.FC<QueueGridProps> = ({ queuedSongs }) => {
  const playingSource = useObservable(() => audioQueue.playingSource);
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const [songTags, setSongTags] = useState<SongTag[]>([]);
  const [color, setColor] = useState('#000000');
  const [name, setName] = useState('');
  const [order, setOrder] = useState(1);
  const [tagId, setTagId] = useState<number | null>(null);

  const width = 200;

  useEffect(() => {
    getJson<SongTag[]>('/tags').then(setSongTags);
  }, []);

  const rowRenderer = (props: ListRowProps) => {
    if (props.style.width) {
      props.style.width = (props.style.width as number) - 11;
    }
    props.style.boxShadow =
      queuedSongs[props.index].path === playingSource
        ? 'inset 0 0 2px 2px rgba(var(--intent-success), 0.3)'
        : 'inset 0 -1px 0 rgba(16, 22, 26, 0.3), inset -1px 0 0 rgba(16, 22, 26, 0.3)';

    return (
      <Draggable draggableId={`queue${props.index.toString()}`} index={props.index} key={props.index}>
        {(provided: DraggableProvided, snapshot: DraggableStateSnapshot) => {
          props.style = { ...props.style, ...provided.draggableProps.style };
          return (
            <FlexRow
              ref={provided.innerRef}
              {...provided.draggableProps}
              {...provided.dragHandleProps}
              key={props.key}
              onDoubleClick={() => audioQueue.start(queuedSongs[props.index].path)}
              style={{ ...provided.draggableProps.style, ...props.style, width: 180 }}
            >
              <div style={{ paddingLeft: 10, fontSize: 12, width: 45 }}>
                {queuedSongs[props.index].path === playingSource ? (
                  <Icon icon='volume-up' style={{ color: 'rgba(var(--intent-success), 1)' }} />
                ) : (
                  <Text ellipsize>{props.index + 1}</Text>
                )}
              </div>
              <FlexCol center={false} style={{ width: 135 }}>
                <Text ellipsize>{queuedSongs[props.index].name}</Text>
                <Text ellipsize className='secondary-text'>
                  {queuedSongs[props.index].album}
                </Text>
                <Text ellipsize className='secondary-text'>
                  {queuedSongs[props.index].artist}
                </Text>
              </FlexCol>
            </FlexRow>
          );
        }}
      </Draggable>
    );
  };

  const headerRowRenderer = (props: TableHeaderRowProps) => {
    props.style.margin = 0;
    props.style.padding = 0;
    return defaultHeaderRowRenderer(props);
  };

  const addTag = () => {
    setName('');
    setOrder(1);
    setColor('#000000');
    setOrder(1);
    setTagId(null);
    setIsPopupOpen(true);
  };

  return (
    <>
      <div style={{ maxWidth: width, paddingLeft: 5 }}>
        <div style={{ minHeight: 10, background: 'rgba(var(--background-secondary), 1)', minWidth: width + 5 }} />
        <FlexCol
          style={{
            fontSize: 16,
            background: 'rgba(var(--background-secondary), 1)',
            paddingBottom: 5,
            minWidth: width + 5,
          }}
        >
          <FlexRow style={{ fontWeight: 700 }}>
            <div style={{ flex: 1 }} />
            Tags
            <Button minimal small style={{ marginLeft: 5, padding: 0 }} onClick={addTag}>
              <Icon iconSize={14} icon='add' style={{ paddingBottom: 1, paddingRight: 1 }} />
            </Button>
          </FlexRow>
        </FlexCol>
        <div
          style={{
            height: (window.innerHeight - 180) / 2,
            overflowY: 'auto',
            background: 'rgba(var(--background-secondary), 1)',
          }}
        >
          {songTags.map((s, i) => {
            return (
              <Droppable droppableId={`tag-${s.id}`} key={i}>
                {(droppableProvided: DroppableProvided, snapshot: DroppableStateSnapshot) => {
                  if (snapshot.isDraggingOver) {
                    console.log('dragging', s.name);
                  }
                  const color = snapshot.isDraggingOver ? hexToRgb(theme.intentPrimary) : hexToRgb(s.color);

                  const editTag = () => {
                    setName(s.name);
                    setOrder(s.order);
                    setColor(s.color);
                    setTagId(s.id);
                    setIsPopupOpen(true);
                  };

                  const deleteTag = async () => {
                    await deleteJson(`/tags/${s.id}`);
                    getJson<SongTag[]>('/tags').then(setSongTags);
                    toastSuccess();
                  };

                  return (
                    <>
                      <div
                        {...droppableProvided.droppableProps}
                        style={{ paddingLeft: 5, paddingBottom: 5 }}
                        ref={droppableProvided.innerRef}
                      >
                        <Tag
                          minimal
                          style={{
                            border: `1px solid rgba(${color}, 0.25)`,
                            backgroundColor: `rgba(${color}, 0.15)`,
                            color: `rgba(${color}, 1)`,
                            boxShadow: snapshot.isDraggingOver
                              ? 'inset 0 0 8px 8px rgba(var(--intent-primary), 0.6)'
                              : undefined,
                          }}
                        >
                          {
                            <FlexRow>
                              <FlexCol>
                                <Popover
                                  content={
                                    <Menu style={{ minWidth: 100 }}>
                                      <MenuItem icon='edit' text='Edit' onClick={editTag} />
                                      <MenuItem icon='delete' text='Delete' onClick={deleteTag} />
                                    </Menu>
                                  }
                                >
                                  <Button minimal small style={{ minHeight: 20, minWidth: 20, marginRight: 2 }}>
                                    <Icon iconSize={12} icon='edit' style={{ paddingBottom: 2 }} />
                                  </Button>
                                </Popover>
                              </FlexCol>
                              <Text ellipsize className='tag-text'>
                                {s.name}
                              </Text>
                              <div style={{ color: 'rgba(var(--text-secondary), 0.9)' }}>23</div>
                            </FlexRow>
                          }
                        </Tag>
                      </div>
                    </>
                  );
                }}
              </Droppable>
            );
          })}
        </div>

        <div style={{ minHeight: 10, background: 'rgba(var(--background-secondary), 1)', minWidth: width }} />
        <FlexCol
          style={{
            fontSize: 16,
            fontWeight: 700,
            background: 'rgba(var(--background-secondary), 1)',
            paddingBottom: 5,
            minWidth: width,
          }}
        >
          Now Playing
        </FlexCol>
        <Droppable
          droppableId='queueGrid'
          mode='virtual'
          renderClone={(provided: DraggableProvided, snapshot: DraggableStateSnapshot, rubric: DraggableRubric) => {
            return <div />;
          }}
        >
          {(droppableProvided: DroppableProvided, snapshot: DroppableStateSnapshot) => {
            return (
              <List
                ref={ref => {
                  const domRef = ReactDOM.findDOMNode(ref);
                  if (domRef instanceof HTMLElement) {
                    droppableProvided.innerRef(domRef as HTMLElement);
                  }
                }}
                width={width - 5}
                height={(window.innerHeight - 180) / 2}
                rowHeight={70}
                disableHeader={true}
                headerHeight={25}
                headerRowRenderer={headerRowRenderer}
                rowCount={queuedSongs.length}
                //rowGetter={(props) => queuedSongs[index]}
                rowRenderer={rowRenderer}
                style={{ background: 'rgba(var(--background-secondary), 1)' }}
                {...droppableProvided.droppableProps}
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
              </List>
            );
          }}
        </Droppable>
      </div>
      <AddEditTag
        isOpen={isPopupOpen}
        setIsOpen={setIsPopupOpen}
        setSongTags={setSongTags}
        color={color}
        setColor={setColor}
        name={name}
        setName={setName}
        order={order}
        setOrder={setOrder}
        tagId={tagId}
      />
    </>
  );
};
