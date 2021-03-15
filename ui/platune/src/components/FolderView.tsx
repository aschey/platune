import { Alert, Button, Intent } from '@blueprintjs/core';
import React, { useCallback, useEffect, useState } from 'react';
import { toastSuccess } from '../appToaster';
import { getJson, putJson } from '../fetchUtil';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { FolderPicker } from './FolderPicker';
import { SelectedFolders } from './SelectedFolders';

interface FolderViewProps {
  width: number;
  height: number;
  panelWidth: number;
  dividerWidth: number;
  buttonHeight: number;
  buttonPanelHeight: number;
  rows: string[];
  setRows: (rows: string[]) => void;
  setOriginalRows: (rows: string[]) => void;
}

export const FolderView: React.FC<FolderViewProps> = ({
  width,
  height,
  panelWidth,
  dividerWidth,
  buttonHeight,
  buttonPanelHeight,
  rows,
  setRows,
  setOriginalRows,
}) => {
  const [selected, setSelected] = useState<string>('');
  const [errorText, setErrorText] = useState<string>('');

  const refreshFolders = useCallback(
    () =>
      getJson<string[]>('/configuredFolders').then(folders => {
        setRows([...folders]);
        setOriginalRows([...folders]);
      }),
    [setRows, setOriginalRows]
  );

  useEffect(() => {
    refreshFolders();
    return () => setRows([]);
  }, [refreshFolders, setRows]);

  const addFolderClick = () => {
    setRows([...rows, selected]);
  };

  const saveFoldersClick = async () => {
    try {
      await putJson<void>('/updateFolders', { folders: rows });
      setOriginalRows([...rows]);
      toastSuccess();
    } catch (e) {
      setErrorText(e.message);
    }
  };

  const revertClick = () => {
    refreshFolders();
  };

  return (
    <>
      <FlexRow center={false} style={{ alignItems: 'top', alignSelf: 'center', width, height }}>
        <FlexCol center={false} style={{ width: panelWidth }}>
          <div style={{ height: height - buttonPanelHeight }}>
            <SelectedFolders rows={rows} setRows={setRows} width={panelWidth} />
          </div>
          <FlexRow>
            <Button
              intent={Intent.SUCCESS}
              icon='floppy-disk'
              text='Save'
              style={{ height: buttonHeight }}
              onClick={saveFoldersClick}
            />
            <div style={{ margin: dividerWidth }} />
            <Button
              intent={Intent.WARNING}
              icon='undo'
              text='Revert'
              style={{ height: buttonHeight }}
              onClick={revertClick}
            />
          </FlexRow>
        </FlexCol>
        <div style={{ width: dividerWidth }} />
        <FlexCol center={false} style={{ width: panelWidth }}>
          <div style={{ height: height - buttonPanelHeight }}>
            <FolderPicker setSelected={setSelected} />
          </div>
          <FlexRow>
            <Button
              intent={Intent.PRIMARY}
              onClick={addFolderClick}
              icon='add'
              text='Add'
              style={{ height: buttonHeight }}
            />
          </FlexRow>
        </FlexCol>
      </FlexRow>
      <Alert intent={Intent.DANGER} isOpen={errorText.length > 0} onClose={() => setErrorText('')}>
        {errorText}
      </Alert>
    </>
  );
};
