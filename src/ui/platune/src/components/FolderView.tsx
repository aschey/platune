import React, { useState, useEffect, useCallback } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import {
  Button,
  ITreeNode,
  Tooltip,
  Position,
  Icon,
  Classes,
  Intent,
  Toaster,
  Toast,
  ButtonGroup,
  Divider,
  Alert,
} from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson, putJson } from '../fetchUtil';
import { SelectedFolders } from './SelectedFolders';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';
import { toastSuccess } from '../appToaster';

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

  const cellRenderer = (rowIndex: number) => {
    return <Cell>{rows[rowIndex]}</Cell>;
  };

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
