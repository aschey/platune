import React, { useEffect, useState } from 'react';
import { Dialog } from './Dialog';
import { SketchPicker, ChromePicker, ColorResult, RGBColor } from 'react-color';
import reactCSS from 'reactcss';
import { InputGroup, FormGroup, ControlGroup, Button, Intent, NumericInput } from '@blueprintjs/core';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { DirtyCheck } from './DirtyCheck';
import { getJson, postJson, putJson } from '../fetchUtil';
import { toastSuccess } from '../appToaster';
import { SongTag } from '../models/songTag';
import { formatRgb } from '../util';
import { EditSongTag } from '../models/editSongTag';
import { Song } from '../models/song';
import { useAppDispatch } from '../state/store';
import { fetchSongs } from '../state/songs';
import { addEditTag, fetchTags } from '../state/songs';
import { batch } from 'react-redux';
import { useThemeContext } from '../state/themeContext';

interface AddEditTagProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
  tag: EditSongTag;
  setTag: (tag: EditSongTag) => void;
}
export const AddEditTag: React.FC<AddEditTagProps> = ({ isOpen, setIsOpen, tag, setTag }) => {
  const [showPicker, setShowPicker] = useState(false);
  const dispatch = useAppDispatch();
  const { themeVal } = useThemeContext();

  const onSave = async () => {
    setIsOpen(false);
    batch(async () => {
      await dispatch(addEditTag(tag));
      dispatch(fetchSongs());
    });

    toastSuccess();
  };

  return (
    <Dialog
      style={{ width: 300, height: 250 }}
      icon='add'
      title={tag.id === null ? 'New Tag' : 'Edit Tag'}
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
            value={tag.name}
            style={{ maxWidth: 175 }}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => setTag({ ...tag, name: e.target.value })}
          />
        </FormGroup>
        <FormGroup label='Order' labelFor='order' inline>
          <NumericInput
            id='order'
            placeholder='Order'
            style={{ maxWidth: 60 }}
            defaultValue={1}
            value={tag.order}
            onValueChange={(_, strValue) => {
              const numericValue = parseInt(strValue);
              setTag({ ...tag, order: isNaN(numericValue) ? 1 : numericValue });
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
                background: `rgb(${tag.color})`,
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
                onClick={() => setShowPicker(false)}
              />
              <SketchPicker
                color={{
                  r: parseInt(tag.color.split(',')[0]),
                  g: parseInt(tag.color.split(',')[1]),
                  b: parseInt(tag.color.split(',')[2]),
                }}
                disableAlpha={true}
                onChange={newColor => setTag({ ...tag, color: formatRgb(newColor.rgb) })}
                presetColors={themeVal.suggestedTagColors}
              />
            </div>
          ) : null}
        </FormGroup>
      </ControlGroup>
      <FlexCol>
        <FlexRow>
          <Button icon='saved' intent={Intent.SUCCESS} style={{ marginRight: 5, width: 80 }} onClick={onSave}>
            Save
          </Button>
          <Button
            icon='undo'
            intent={Intent.WARNING}
            style={{ marginLeft: 5, width: 80 }}
            onClick={() => setIsOpen(false)}
          >
            Cancel
          </Button>
        </FlexRow>
      </FlexCol>
    </Dialog>
  );
};
