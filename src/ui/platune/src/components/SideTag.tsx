import { Button, Icon, Menu, MenuItem, Popover, Tag, Text } from '@blueprintjs/core';
import React, { useState } from 'react';
import { toastSuccess } from '../appToaster';
import { deleteJson, getJson } from '../fetchUtil';
import { EditSongTag } from '../models/editSongTag';
import { Search } from '../models/search';
import { SongTag } from '../models/songTag';
import { useAppDispatch } from '../state/store';
import { deleteTag } from '../state/songs';
import { hexToRgb, isLight, shadeColorRgb } from '../themes/colorMixer';
import { theme } from './App';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';

interface SideTagProps {
  tag: SongTag;
  setTag: (tag: EditSongTag) => void;
  setIsPopupOpen: (isPopupOpen: boolean) => void;
  isDraggingOver: boolean;
  isLightTheme: boolean;
  setSelectedSearch: (selectedSearch: Search) => void;
}
export const SideTag: React.FC<SideTagProps> = ({
  tag,
  setTag,
  setIsPopupOpen,
  isDraggingOver,
  isLightTheme,
  setSelectedSearch,
}) => {
  const dispatch = useAppDispatch();

  const [hovered, setHovered] = useState(false);

  const editTag = () => {
    setTag(tag);
    setIsPopupOpen(true);
  };

  const onDeleteTag = async () => {
    dispatch(deleteTag(tag.id));
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
      onMouseOver={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      minimal
      style={{
        border: `1px solid rgba(${color}, 0.25)`,
        backgroundColor: `rgba(${color}, ${hovered ? 0.3 : 0.15})`,
        color: `rgba(${shadeColorRgb(color, isLightTheme ? -50 : 100)}, 1)`,
        boxShadow: isDraggingOver ? `inset 0 0 8px 8px rgba(${color}, 0.6)` : undefined,
        cursor: hovered ? 'pointer' : undefined,
      }}
    >
      {
        <FlexRow>
          <FlexCol>
            <Popover
              content={
                <Menu style={{ minWidth: 100 }}>
                  <MenuItem icon='edit' text='Edit' onClick={editTag} />
                  <MenuItem icon='delete' text='Delete' onClick={onDeleteTag} />
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
