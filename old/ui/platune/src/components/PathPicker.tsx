import { Button, Classes, Colors, Intent, Text } from '@blueprintjs/core';
import React, { useEffect, useState } from 'react';
import { toastSuccess } from '../appToaster';
import { getJson, putJson } from '../fetchUtil';
import { Dir } from '../models/dir';
import { FlexRow } from './FlexRow';
import { FolderPicker } from './FolderPicker';

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
      const dbFound = res.dirs.some(d => d.isFile && d.name.endsWith('platune.db'));
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
    <FlexRow center={false} style={{ alignItems: 'top', alignSelf: 'center', width, height: height }}>
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
        <FlexRow center={false} style={{ margin: 5, marginLeft: 5 }}>
          <Button
            intent={Intent.SUCCESS}
            icon='floppy-disk'
            text='Save'
            style={{ height: buttonHeight }}
            onClick={onSaveClick}
          />
          <div style={{ margin: 5 }} />
          <Button
            intent={Intent.WARNING}
            icon='undo'
            text='Revert'
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
