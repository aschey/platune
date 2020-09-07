import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { SketchPicker, ChromePicker, ColorResult, RGBColor } from 'react-color';
import reactCSS from 'reactcss';
import { InputGroup, FormGroup, ControlGroup, Button, Intent } from '@blueprintjs/core';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';

interface AddEditTagProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}
export const AddEditTag: React.FC<AddEditTagProps> = ({ isOpen, setIsOpen }) => {
  const [color, setColor] = useState<RGBColor>({ r: 241, g: 112, b: 19, a: 1 });
  const [showPicker, setShowPicker] = useState(false);

  return (
    <Dialog
      style={{ width: 300, height: 200 }}
      icon='add'
      title='New Tag'
      isOpen={isOpen}
      onOpening={() => setShowPicker(false)}
      onClose={() => setIsOpen(false)}
      autoFocus
      enforceFocus
      usePortal
    >
      <ControlGroup vertical onClick={() => setShowPicker(false)}>
        <FormGroup label='Tag Name' labelFor='tagName' inline>
          <InputGroup id='tagName' placeholder='Enter tag name' />
        </FormGroup>

        <FormGroup label='Color' labelFor='tagColor' inline style={{ alignItems: 'center' }}>
          <div
            id='tagColor'
            style={{
              borderRadius: '1px',
              boxShadow: '0 0 0 1px rgba(0,0,0,.1)',
              display: 'inline-block',
              cursor: 'pointer',
            }}
            onClick={e => {
              e.stopPropagation();
              setShowPicker(!showPicker);
            }}
          >
            <div
              style={{
                width: 36,
                height: 14,
                marginTop: 4,
                borderRadius: 2,
                background: `rgba(${color.r}, ${color.g}, ${color.b}, ${color.a})`,
              }}
            />
          </div>
          {showPicker ? (
            <div
              style={{
                position: 'absolute',
                zIndex: 2,
              }}
            >
              <SketchPicker
                color={color}
                disableAlpha={true}
                onChange={newColor => setColor(newColor.rgb)}
                presetColors={[{ color: '#FF0000', title: 'red' }]}
              />
            </div>
          ) : null}
        </FormGroup>
      </ControlGroup>
      <FlexCol>
        <FlexRow>
          <Button intent={Intent.SUCCESS} style={{ marginRight: 5 }}>
            Save
          </Button>
          <Button intent={Intent.WARNING} style={{ marginLeft: 5 }}>
            Cancel
          </Button>
        </FlexRow>
      </FlexCol>
    </Dialog>
  );
};
