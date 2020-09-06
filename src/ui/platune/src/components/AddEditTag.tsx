import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { SketchPicker, ChromePicker, ColorResult, RGBColor } from 'react-color';
import reactCSS from 'reactcss';

interface AddEditTagProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}
export const AddEditTag: React.FC<AddEditTagProps> = ({ isOpen, setIsOpen }) => {
  const [color, setColor] = useState<RGBColor>({ r: 241, g: 112, b: 19, a: 1 });
  const [showPicker, setShowPicker] = useState(false);

  return (
    <Dialog
      style={{ width: 500, height: 500 }}
      icon='add'
      title='New Tag'
      isOpen={isOpen}
      onClose={() => setIsOpen(false)}
      autoFocus
      enforceFocus
      usePortal
    >
      <div>
        <div
          style={{
            borderRadius: '1px',
            boxShadow: '0 0 0 1px rgba(0,0,0,.1)',
            display: 'inline-block',
            cursor: 'pointer',
          }}
          onClick={() => setShowPicker(!showPicker)}
        >
          <div
            style={{
              width: '36px',
              height: '14px',
              borderRadius: '2px',
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
            <div
              style={{
                position: 'fixed',
                top: '0px',
                right: '0px',
                bottom: '0px',
                left: '0px',
              }}
              onClick={() => setShowPicker(false)}
            />
            <SketchPicker
              color={color}
              disableAlpha={true}
              onChange={newColor => setColor(newColor.rgb)}
              presetColors={[{ color: '#FF0000', title: 'red' }]}
            />
          </div>
        ) : null}
      </div>
    </Dialog>
  );
};
