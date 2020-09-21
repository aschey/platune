import { Button, Icon, Intent, Tag, Text } from '@blueprintjs/core';
import { lighten } from 'color-blend';
import React, { useState } from 'react';
import { shadeColor, shadeColorRgb } from '../themes/colorMixer';
import { FlexRow } from './FlexRow';

interface GridTagProps {
  name: string;
  color: string;
  isLightTheme: boolean;
}
export const GridTag: React.FC<GridTagProps> = ({ name, color, isLightTheme }) => {
  const [showDelete, setShowDelete] = useState(false);

  return (
    <Tag
      minimal
      style={{
        height: 20,
        marginTop: 2,
        marginRight: 5,
        border: `1px solid rgba(${color}, 0.25)`,
        backgroundColor: `rgba(${color}, 0.15)`,
        color: `rgba(${shadeColorRgb(color, isLightTheme ? -50 : 200)}, 1)`,
      }}
      onMouseEnter={() => setShowDelete(true)}
      onMouseLeave={() => setShowDelete(false)}
    >
      <FlexRow>
        {showDelete ? (
          <Button minimal small style={{ minHeight: 20, minWidth: 20, marginRight: 2 }}>
            <Icon iconSize={12} icon='delete' style={{ paddingBottom: 2 }} />
          </Button>
        ) : null}
        <Text ellipsize>{name}</Text>
      </FlexRow>
    </Tag>
  );
};
