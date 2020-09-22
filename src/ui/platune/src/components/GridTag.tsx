import { Button, Icon, Intent, Tag, Text } from '@blueprintjs/core';
import { lighten } from 'color-blend';
import React, { useState } from 'react';
import { toastSuccess } from '../appToaster';
import { putJson } from '../fetchUtil';
import { GridTagRes } from '../models/gridTagRes';
import { shadeColor, shadeColorRgb } from '../themes/colorMixer';
import { FlexRow } from './FlexRow';

interface GridTagProps {
  tag: GridTagRes;
  isLightTheme: boolean;
  songId: number;
}
export const GridTag: React.FC<GridTagProps> = ({ tag, isLightTheme, songId }) => {
  const [showDelete, setShowDelete] = useState(false);
  const { color, name, id } = tag;
  const removeTag = async () => {
    await putJson(`/tags/${id}/removeSongs`, [songId]);
    toastSuccess();
  };
  return (
    <Tag
      minimal
      className='grid-tag'
      style={{
        height: 20,
        marginTop: 2,
        marginRight: 5,
        border: `1px solid rgba(${color}, 0.25)`,
        backgroundColor: `rgba(${color}, 0.15)`,
        color: `rgba(${shadeColorRgb(color, isLightTheme ? -50 : 100)}, 1)`,
      }}
      onMouseEnter={() => setShowDelete(true)}
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
