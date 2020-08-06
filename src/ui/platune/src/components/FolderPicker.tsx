import React, { useState, useEffect, useCallback } from 'react';
import logo from './logo.svg';
import { Icon, Intent, ITreeNode, Position, Tooltip, Tree, Classes, IconName } from '@blueprintjs/core';
import { getJson } from '../fetchUtil';
import { Dir } from '../models/dir';
import { SelectedFolder } from '../models/selectedFolder';

interface FolderPickerProps {
  setSelected(folder: string): void;
}

export const FolderPicker: React.FC<FolderPickerProps> = ({ setSelected }: FolderPickerProps) => {
  const [nodes, setNodes] = useState<ITreeNode[]>([]);
  const [delim, setDelim] = useState<string>('');

  const getFullPath = useCallback(
    (node: ITreeNode): string => {
      let path = node.label.toString();
      if (delim === '\\' && node.nodeData === undefined) {
        return `${path}\\`;
      }
      while (node.nodeData !== undefined) {
        let parentNode = node.nodeData as ITreeNode;
        let parentDir = parentNode.label === delim ? '' : parentNode.label;
        path = `${parentDir}${delim}${path}`;
        node = parentNode;
      }
      return path;
    },
    [delim]
  );

  const getNodes = useCallback(
    async (
      path: string,
      rootNode: ITreeNode | undefined,
      shouldExpand: boolean | null = null
    ): Promise<ITreeNode<{}>[]> => {
      let dirsResponse = await getJson<{ dirs: Array<Dir> }>(path);
      const isExpanded = shouldExpand ?? dirsResponse.dirs.length === 1;
      const icon: IconName = isExpanded ? 'folder-open' : 'folder-close';
      let _nodes = dirsResponse.dirs
        .filter(dir => !dir.isFile)
        .map(
          (dir): ITreeNode => {
            let node: ITreeNode = {
              id: '',
              hasCaret: true,
              icon: icon,
              label: dir.name,
              isExpanded,
              childNodes: [],
              nodeData: rootNode,
            };
            node.id = getFullPath(node);
            return node;
          }
        );
      return _nodes;
    },
    [getFullPath]
  );

  const updateNodes = useCallback(
    async (rootNode: ITreeNode, nodes: ITreeNode[]): Promise<void> => {
      const childNodes = await getNodes(`/dirs?dir=${rootNode.id}`, rootNode, false);
      rootNode.childNodes = childNodes;
      setNodes([...nodes]);
    },
    [getNodes]
  );

  useEffect(() => {
    getJson<boolean>('/isWindows').then(isWindows => setDelim(isWindows ? '\\' : '/'));
  }, []);

  useEffect(() => {
    if (delim === '') {
      return;
    }
    getNodes('/dirsInit', undefined).then(async _nodes => {
      for (let node of _nodes) {
        await updateNodes(node, _nodes);
      }
    });
  }, [delim, getNodes, updateNodes]);

  const handleNodeClick = (nodeData: ITreeNode, _nodePath: number[], e: React.MouseEvent<HTMLElement>) => {
    const originallySelected = nodeData.isSelected;
    // if (!e.shiftKey) {
    forEachNode(nodes, n => (n.isSelected = false));
    // }
    nodeData.isSelected = originallySelected == null ? true : !originallySelected;
    setNodes([...nodes]);
    setSelected(nodeData.id as string);
  };

  const handleNodeCollapse = (nodeData: ITreeNode) => {
    nodeData.isExpanded = false;
    setNodes([...nodes]);
  };

  const handleNodeExpand = async (nodeData: ITreeNode) => {
    nodeData.isExpanded = true;
    nodeData.icon = 'folder-open';
    await updateNodes(nodeData, nodes);
  };

  const forEachNode = (nodes: ITreeNode[] | undefined, callback: (node: ITreeNode) => void) => {
    if (nodes == null) {
      return;
    }

    for (const node of nodes) {
      callback(node);
      forEachNode(node.childNodes, callback);
    }
  };

  return (
    <Tree
      contents={nodes}
      onNodeClick={handleNodeClick}
      onNodeCollapse={handleNodeCollapse}
      onNodeExpand={handleNodeExpand}
      className={`{Classes.ELEVATION_0} Expand-Column bp3-table-container scroll`}
    />
  );
};
