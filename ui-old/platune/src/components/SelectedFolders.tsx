import { Button, Intent } from '@blueprintjs/core';
import { Cell, Column, Table, TruncatedFormat, TruncatedPopoverMode } from '@blueprintjs/table';
import React from 'react';

interface SelectedFoldersProps {
  rows: string[];
  setRows: (rows: string[]) => void;
  width: number;
}

export const SelectedFolders: React.FC<SelectedFoldersProps> = ({ rows, setRows, width }: SelectedFoldersProps) => {
  const deleteRowWidth = 30;

  const deleteClick = (rowIndex: number) => {
    const newRows = rows.filter((r, i) => i !== rowIndex);
    setRows([...newRows]);
  };

  const pathRenderer = (rowIndex: number) => (
    <Cell style={{ fontSize: 14 }}>
      <TruncatedFormat detectTruncation showPopover={TruncatedPopoverMode.WHEN_TRUNCATED}>
        {rows[rowIndex]}
      </TruncatedFormat>
    </Cell>
  );

  const deleteRenderer = (rowIndex: number) => (
    <Cell style={{ padding: '0 3px' }}>
      <Button intent={Intent.DANGER} icon='delete' onClick={() => deleteClick(rowIndex)} minimal small />
    </Cell>
  );

  return (
    <Table
      numRows={rows.length}
      columnWidths={[width - deleteRowWidth * 2 - 10, deleteRowWidth]}
      rowHeights={rows.map(() => 25)}
    >
      <Column name='Path' cellRenderer={pathRenderer} />
      <Column name='' cellRenderer={deleteRenderer} />
    </Table>
  );
};
