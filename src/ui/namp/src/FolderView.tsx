import React, { useState, useEffect } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson } from './fetchUtil';

export const FolderView: React.FC<{}> = () => {
    const [rows, setRows] = useState<Array<string>>([]);
    
    useEffect(() => {
      getJson<Array<string>>('/configuredFolders').then(setRows);
    }, []);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };
    return (
        <div style={{display: 'flex', alignItems: 'center'}}>
        <Table numRows={rows.length}>
            <Column name="Path" cellRenderer={cellRenderer}/>
        </Table>
        <FolderPicker nodes={INITIAL_STATE} homeDir='/home/aschey'/>
        </div>
    )
}

const INITIAL_STATE: ITreeNode[] = [
  {
      id: 0,
      hasCaret: true,
      icon: "folder-close",
      label: "Folder 0",
  },
  {
      id: 1,
      icon: "folder-close",
      isExpanded: true,
      label: (
          <Tooltip content="I'm a folder <3" position={Position.RIGHT}>
              Folder 1
          </Tooltip>
      ),
      childNodes: [
          {
              id: 2,
              icon: "document",
              label: "Item 0",
              secondaryLabel: (
                  <Tooltip content="An eye!">
                      <Icon icon="eye-open" />
                  </Tooltip>
              ),
          },
          {
              id: 3,
              icon: <Icon icon="tag" intent={Intent.PRIMARY} className={Classes.TREE_NODE_ICON} />,
              label: "Organic meditation gluten-free, sriracha VHS drinking vinegar beard man.",
          },
          {
              id: 4,
              hasCaret: true,
              icon: "folder-close",
              label: (
                  <Tooltip content="foo" position={Position.RIGHT}>
                      Folder 2
                  </Tooltip>
              ),
              childNodes: [
                  { id: 5, label: "No-Icon Item" },
                  { id: 6, icon: "tag", label: "Item 1" },
                  {
                      id: 7,
                      hasCaret: true,
                      icon: "folder-close",
                      label: "Folder 3",
                      childNodes: [
                          { id: 8, icon: "document", label: "Item 0" },
                          { id: 9, icon: "tag", label: "Item 1" },
                      ],
                  },
              ],
          },
      ],
  },
  {
      id: 2,
      hasCaret: true,
      icon: "folder-close",
      label: "Super secret files",
      disabled: true,
  },
];