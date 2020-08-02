import React, { useState, useEffect, Children } from 'react';
import { Alert, Intent, IDialogProps, TextArea, Button, Text, Classes, Tooltip, Colors } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { getJson, putJson } from '../fetchUtil';
import { Dir } from '../models/dir';
import { toastSuccess } from '../appToaster';

interface PathPickerProps {
  width: number;
  height: number;
  panelWidth: number;
  dividerWidth: number;
  marginBottom: number;
  buttonHeight: number;
  setOriginalPath: (originalPath: string) => void;
  path: string;
  setPath: (path: string) => void;
}

export const PathPicker: React.FC<PathPickerProps> = ({
  width,
  height,
  buttonHeight,
  setOriginalPath,
  path,
  setPath,
  panelWidth,
  dividerWidth,
}) => {
  const PLACEHOLDER = 'placeholder';
  const [databaseFound, setDatabaseFound] = useState<boolean>(false);
  const [displayText, setDisplayText] = useState<string>(PLACEHOLDER);
  useEffect(() => {
    getJson<{ name: string }>('/getDbPath').then(res => {
      setOriginalPath(res.name);
      setPath(res.name);
    });
  }, [setOriginalPath, setPath]);

  useEffect(() => {
    if (path === '') {
      return;
    }
    getJson<{ dirs: Dir[] }>(`/dirs?dir=${path}`).then(res => {
      const dbFound = res.dirs.some(d => d.isFile && d.name.endsWith('namp.db'));
      setDatabaseFound(dbFound);
      setDisplayText(dbFound ? '* Existing database found' : '* Existing database not found');
    });
    return () => setDisplayText(PLACEHOLDER);
  }, [path, setDatabaseFound]);

  const onSaveClick = async () => {
    await putJson<{}>('/updateDbPath', { dir: path });
    setOriginalPath(path);
    toastSuccess();
  };

  const onRevertClick = () => {
    getJson<{ name: string }>('/getDbPath').then(res => {
      setOriginalPath(res.name);
      setPath(res.name);
    });
  };

  return (
    <FlexRow style={{ alignItems: 'top', alignSelf: 'center', width, height: height }}>
      <div style={{ width: panelWidth }} className={'bp3-table-container'}>
        <div style={{ margin: 5 }}>
          <Text ellipsize className={Classes.INPUT}>
            {path}
          </Text>
          <div style={{ color: databaseFound ? Colors.GREEN2 : Colors.ORANGE2, paddingTop: 5, paddingLeft: 0 }}>
            <Text className={displayText === PLACEHOLDER ? 'bp3-skeleton' : ''}>{displayText} </Text>
          </div>
        </div>
        <div style={{ height: 5 }} />
        <FlexRow style={{ margin: 5, marginLeft: 5 }}>
          <Button
            intent={Intent.SUCCESS}
            icon="floppy-disk"
            text="Save"
            style={{ height: buttonHeight }}
            onClick={onSaveClick}
          />
          <div style={{ margin: 5 }} />
          <Button
            intent={Intent.WARNING}
            icon="undo"
            text="Revert"
            style={{ height: buttonHeight }}
            onClick={onRevertClick}
          />
        </FlexRow>
      </div>
      <div style={{ width: dividerWidth }} />
      <div style={{ width: panelWidth, height: height }}>
        <FolderPicker setSelected={setPath} />
      </div>
    </FlexRow>
  );
};
