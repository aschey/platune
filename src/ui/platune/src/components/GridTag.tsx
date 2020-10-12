import { Button, Icon, Intent, Tag, Text } from '@blueprintjs/core';
import { lighten } from 'color-blend';
import React, { useState } from 'react';
import { batch } from 'react-redux';
import { toastSuccess } from '../appToaster';
import { putJson } from '../fetchUtil';
import { GridTagRes } from '../models/gridTagRes';
import { useAppDispatch } from '../state/store';
import { removeSongsFromTag } from '../state/songs';
import { shadeColor, shadeColorRgb } from '../themes/colorMixer';
import { FlexRow } from './FlexRow';
import { formatCssVar } from '../util';

interface GridTagProps {
  tag: GridTagRes;
  isLightTheme: boolean;
  songId: number;
  useCustomColors: boolean;
}
export const GridTag: React.FC<GridTagProps> = ({ tag, isLightTheme, songId, useCustomColors }) => {
  const [showDelete, setShowDelete] = useState(false);
  const dispatch = useAppDispatch();
  const { color, name, id } = tag;
  const removeTag = async () => {
    dispatch(removeSongsFromTag({ tagId: id, songIds: [songId] }));

    toastSuccess();
  };
  const tagVar = formatCssVar(tag.name);
  return (
    <Tag
      minimal
      style={{
        height: 20,
        marginTop: 2,
        marginRight: 5,
        border: useCustomColors ? `1px solid rgba(var(--tag-bg-${tagVar}), 0.5)` : `1px solid rgba(${color}, 0.25)`,
        backgroundColor: useCustomColors ? `rgba(var(--tag-bg-${tagVar}), 0.3)` : `rgba(${color}, 0.15)`,
        color: useCustomColors
          ? `rgba(var(--tag-fg-${tagVar}), 1)`
          : `rgba(${shadeColorRgb(color, isLightTheme ? -50 : 100)}, 1)`,
      }}
      onMouseOver={() => setShowDelete(true)}
      onMouseLeave={() => setShowDelete(false)}
    >
      <FlexRow>
        {showDelete ? (
          <Button minimal small style={{ minHeight: 20, minWidth: 20, marginRight: 2 }} onClick={removeTag}>
            <Icon iconSize={12} icon='delete' style={{ paddingBottom: 2 }} />
          </Button>
        ) : null}
        <Text ellipsize>{name}</Text>
      </FlexRow>
    </Tag>
  );
};
