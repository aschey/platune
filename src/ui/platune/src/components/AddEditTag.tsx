import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { SketchPicker, ChromePicker, ColorResult, RGBColor } from 'react-color';
import reactCSS from 'reactcss';
import { InputGroup, FormGroup, ControlGroup, Button, Intent, NumericInput } from '@blueprintjs/core';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { DirtyCheck } from './DirtyCheck';

interface AddEditTagProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}
export const AddEditTag: React.FC<AddEditTagProps> = ({ isOpen, setIsOpen }) => {
  const [color, setColor] = useState('#000000');
  const [showPicker, setShowPicker] = useState(false);
  const [name, setName] = useState('');
  const [order, setOrder] = useState(1);

  return (
    <Dialog
      style={{ width: 300, height: 250 }}
      icon='add'
      title='New Tag'
      isOpen={isOpen}
      onOpening={() => setShowPicker(false)}
      onClose={() => setIsOpen(false)}
      autoFocus
      enforceFocus
    >
      <ControlGroup vertical>
        <FormGroup label='Tag Name' labelFor='tagName' inline>
          <InputGroup
            id='tagName'
            placeholder='Enter a tag name'
            value={name}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
          />
        </FormGroup>
        <FormGroup label='Order' labelFor='order' inline>
          <NumericInput
            id='order'
            placeholder='Order'
            style={{ maxWidth: 60 }}
            defaultValue={1}
            value={order}
            onValueChange={(_, strValue) => {
              const numericValue = parseInt(strValue);
              setOrder(isNaN(numericValue) ? 1 : numericValue);
            }}
          />
        </FormGroup>
        <FormGroup label='Color' labelFor='tagColor' inline style={{ alignItems: 'center' }}>
          <div
            id='tagColor'
            style={{
              borderRadius: '1px',
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
                background: color,
              }}
            />
          </div>
          {showPicker ? (
            <div
              style={{
                position: 'absolute',
                display: 'block',
                zIndex: 2,
              }}
            >
              <div
                style={{
                  position: 'fixed',
                  top: -55,
                  right: 0,
                  bottom: -55,
                  left: 0,
                }}
                onClick={e => {
                  console.log(e.type, e.target);
                  setShowPicker(false);
                }}
              />
              <SketchPicker
                color={color}
                disableAlpha={true}
                onChange={newColor => setColor(newColor.hex)}
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
