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
import { SideTag } from './SideTag';
import { EditSongTag } from '../models/editSongTag';
import { Search } from '../models/search';

interface QueueGridProps {
  queuedSongs: Song[];
  isLightTheme: boolean;
  songTags: SongTag[];
  setSongTags: (songTags: SongTag[]) => void;
  setSongs: (songs: Song[]) => void;
  setSelectedSearch: (selectedSearch: Search | null) => void;
}

export const QueueGrid: React.FC<QueueGridProps> = ({
  queuedSongs,
  isLightTheme,
  songTags,
  setSongTags,
  setSongs,
  setSelectedSearch,
}) => {
  const playingSource = useObservable(() => audioQueue.playingSource);
  const [isPopupOpen, setIsPopupOpen] = useState(false);
  const [tag, setTag] = useState<EditSongTag>({ name: '', order: 1, color: '0,0,0', id: null });

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
    setTag({ id: null, name: '', order: 1, color: '0,0,0' });
    setIsPopupOpen(true);
  };

  return (
    <div style={{ background: 'rgba(var(--background-main), 1)' }}>
      <div style={{ maxWidth: width, paddingLeft: 5 }}>
        <div style={{ minHeight: 10, background: 'rgba(var(--background-main), 1)' }} />
        <FlexCol
          style={{
            fontSize: 16,
            background: 'rgba(var(--background-main), 1)',
            paddingBottom: 5,
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
            borderRadius: 10,
            background: 'rgba(var(--background-secondary), 1)',
            paddingTop: 5,
          }}
        >
          {songTags.map((s, i) => {
            return (
              <Droppable droppableId={`tag-${s.id}`} key={i}>
                {(droppableProvided: DroppableProvided, snapshot: DroppableStateSnapshot) => {
                  if (snapshot.isDraggingOver) {
                    console.log('dragging', s.name);
                  }

                  return (
                    <>
                      <div
                        {...droppableProvided.droppableProps}
                        style={{ paddingLeft: 5, paddingBottom: 5 }}
                        ref={droppableProvided.innerRef}
                      >
                        <SideTag
                          isDraggingOver={snapshot.isDraggingOver}
                          tag={s}
                          setTag={setTag}
                          setIsPopupOpen={setIsPopupOpen}
                          setSongTags={setSongTags}
                          isLightTheme={isLightTheme}
                          setSelectedSearch={setSelectedSearch}
                        />
                      </div>
                    </>
                  );
                }}
              </Droppable>
            );
          })}
        </div>

        <div style={{ minHeight: 10, background: 'rgba(var(--background-main), 1)' }} />
        <FlexCol
          style={{
            fontSize: 16,
            fontWeight: 700,
            background: 'rgba(var(--background-main), 1)',
            paddingBottom: 5,
          }}
        >
          Now Playing
        </FlexCol>
        {queuedSongs.length > 0 ? (
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
                  height={(window.innerHeight - 180) / 2 - 5}
                  rowHeight={70}
                  disableHeader={true}
                  headerHeight={25}
                  headerRowRenderer={headerRowRenderer}
                  rowCount={queuedSongs.length}
                  rowRenderer={rowRenderer}
                  style={{
                    overflowY: 'scroll',
                    borderRadius: '0 0 10px 10px',
                    background: 'rgba(var(--background-secondary), 1)',
                  }}
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
                    width={width - 60}
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
        ) : (
          <div
            style={{
              background: 'rgba(var(--background-secondary), 1)',
              borderRadius: 10,
              height: (window.innerHeight - 180) / 2 - 5,
              fontSize: 16,
              color: 'rgba(var(--text-secondary), 1)',
            }}
          >
            <div style={{ padding: 10 }}>Drag a song here or press play to start the queue</div>
          </div>
        )}
      </div>
      <AddEditTag
        isOpen={isPopupOpen}
        setIsOpen={setIsPopupOpen}
        setSongTags={setSongTags}
        tag={tag}
        setTag={setTag}
        setSongs={setSongs}
      />
    </div>
  );
};
