import React from 'react';
import { useDrop } from 'react-dnd';
import { Tag as BlueprintTag, Button, Icon, Text } from '@blueprintjs/core';
import { Rgb } from '../models/rgb';
import { formatRgb } from '../util';
import { darken } from 'color-blend';
import { shadeColor, hexToRgbStr } from '../themes/colorMixer';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { Tag } from '../models/tag';

interface SidebarTagProps {
  tag: Tag;
}

export const SidebarTag: React.FC<SidebarTagProps> = ({ tag }) => {
  const { name, color } = tag;
  const [{ isOver, canDrop }, drop] = useDrop({
    accept: 'song',
    drop: e => {
      console.log(e);
    },
    collect: monitor => ({
      isOver: !!monitor.isOver(),
      canDrop: !!monitor.canDrop(),
    }),
  });
  const bg = hexToRgbStr(shadeColor(color, -20));

  return (
    <div ref={drop}>
      <BlueprintTag minimal style={{ color, border: `1px solid rgba(${bg}, 0.25)`, background: `rgba(${bg}, 0.15)` }}>
        {
          <FlexRow>
            <FlexCol>
              <Button minimal small style={{ minHeight: 20, minWidth: 20, marginRight: 2 }}>
                <Icon iconSize={12} icon='edit' style={{ paddingBottom: 2 }} />
              </Button>
            </FlexCol>
            <Text ellipsize className='tag-text'>
              {name}
            </Text>
            <div style={{ color: 'rgba(var(--text-secondary), 0.9)' }}>23</div>
          </FlexRow>
        }
      </BlueprintTag>
    </div>
  );
};
