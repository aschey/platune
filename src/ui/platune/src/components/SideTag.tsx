import { Button, Icon, Menu, MenuItem, Popover, Tag, Text } from '@blueprintjs/core';
import React from 'react';
import { toastSuccess } from '../appToaster';
import { deleteJson, getJson } from '../fetchUtil';
import { EditSongTag } from '../models/editSongTag';
import { Search } from '../models/search';
import { SongTag } from '../models/songTag';
import { hexToRgb, isLight, shadeColorRgb } from '../themes/colorMixer';
import { theme } from './App';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';

interface SideTagProps {
  tag: SongTag;
  setTag: (tag: EditSongTag) => void;
  setIsPopupOpen: (isPopupOpen: boolean) => void;
  setSongTags: (songTags: SongTag[]) => void;
  isDraggingOver: boolean;
  isLightTheme: boolean;
  setSelectedSearch: (selectedSearch: Search) => void;
}
export const SideTag: React.FC<SideTagProps> = ({
  tag,
  setTag,
  setIsPopupOpen,
  setSongTags,
  isDraggingOver,
  isLightTheme,
  setSelectedSearch,
}) => {
  const editTag = () => {
    setTag(tag);
    setIsPopupOpen(true);
  };

  const deleteTag = async () => {
    await deleteJson(`/tags/${tag.id}`);
    getJson<SongTag[]>('/tags').then(setSongTags);
    toastSuccess();
  };

  const color = isDraggingOver ? hexToRgb(theme.intentPrimary).join(',') : tag.color;

  return (
    <Tag
      onDoubleClick={() =>
        setSelectedSearch({
          entryValue: tag.name,
          entryType: 'tag',
          artist: null,
          correlationId: tag.id,
          tagColor: tag.color,
        })
      }
      minimal
      style={{
        border: `1px solid rgba(${color}, 0.25)`,
        backgroundColor: `rgba(${color}, 0.15)`,
        color: `rgba(${shadeColorRgb(color, isLightTheme ? -50 : 100)}, 1)`,
        boxShadow: isDraggingOver ? `inset 0 0 8px 8px rgba(${color}, 0.6)` : undefined,
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
            {tag.name}
          </Text>
          <div style={{ color: 'rgba(var(--text-secondary), 0.9)' }}>{tag.songCount}</div>
        </FlexRow>
      }
    </Tag>
  );
};
