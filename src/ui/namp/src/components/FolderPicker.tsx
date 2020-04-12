import React, { useState, useEffect } from 'react';
import logo from './logo.svg';
import '../css/App.css';
import { Icon, Intent, ITreeNode, Position, Tooltip, Tree, Classes } from "@blueprintjs/core";
import { getJson } from '../fetchUtil';

interface FolderPickerProps {
    setSelected(folder: string): void;
    width: number;
    height: number;
}

export const FolderPicker: React.FC<FolderPickerProps> = ({ setSelected, width, height }: FolderPickerProps) => {
  const [nodes, setNodes] = useState<ITreeNode[]>([]);
  const [delim, setDelim] = useState<string>('');

  useEffect(() => {
    getJson<boolean>('/isWindows').then(isWindows => setDelim(isWindows ? '\\' : '/'));
  }, []);

  useEffect(() => {
    getNodes('/dirsInit', undefined).then(async _nodes => {
        for (let node of _nodes) {
            await updateNodes(node, _nodes);
        }
    });    
  }, [delim]);

  const getNodes = async (path: string, rootNode: ITreeNode | undefined, shouldExpand: boolean | null = null): Promise<ITreeNode<{}>[]> => {
    let dirsResponse = await getJson<{dirs: Array<string>}>(path);
    const isExpanded = shouldExpand ?? dirsResponse.dirs.length === 1;
    let _nodes = dirsResponse.dirs.map((dir): ITreeNode => {
        let node: ITreeNode = {
            id: '',
            hasCaret: true,
            icon: 'folder-close',
            label: dir,
            isExpanded,
            childNodes: [],
            nodeData: rootNode,
        }
        node.id = getFullPath(node);
        return node;
    });
    return _nodes;
  }

  const updateNodes = async (rootNode: ITreeNode, nodes: ITreeNode[]): Promise<void> => {
    const childNodes = await getNodes(`/dirs?dir=${rootNode.id}`, rootNode, false);
    rootNode.childNodes = childNodes;
    setNodes([...nodes]);
  }
  
  const getFullPath = (node: ITreeNode): string => {
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
  }

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
  }


  return (
        <Tree
            contents={nodes}
            onNodeClick={handleNodeClick}
            onNodeCollapse={handleNodeCollapse}
            onNodeExpand={handleNodeExpand}
            className={`{Classes.ELEVATION_0} Expand-Column ${Classes.getClassNamespace()}-table-container scroll`}
        />
  );
}
