import React, { useState, useEffect } from 'react';
import logo from './logo.svg';
import '../css/App.css';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { getJson } from '../fetchUtil';
import { start } from 'repl';

interface FolderPickerProps {
    setSelected(folder: string): void;
    width: string;
    height: string;
}

export const FolderPicker: React.FC<FolderPickerProps> = ({ setSelected, width, height }: FolderPickerProps) => {
  const [nodes, setNodes] = useState<ITreeNode[]>([]);
  const [id, setId] = useState<number>(0);
  const [delim, setDelim] = useState<string>('');

  useEffect(() => {
    getJson('/isWindows').then(isWindows =>
        { 
            let _delim = isWindows ? '\\' : '/';
            setDelim(_delim);
            const rootNode: ITreeNode = {
                id,
                hasCaret: true,
                icon: 'folder-close',
                label: _delim,
                isExpanded: true,
                childNodes: [],
                nodeData: undefined
            }
            updateNodes(_delim, rootNode, [rootNode], id + 1);
        });
    
  }, []);

  const updateNodes = (path: string, rootNode: ITreeNode, nodes: ITreeNode[], startId: number) => {
    getJson<{dirs: Array<string>}>(`/dirs?dir=${path}`).then(dirsResponse => {
        let _id = startId;
        const dirs = dirsResponse.dirs;
        const childNodes = dirs.map((d, i): ITreeNode => {
            let node: ITreeNode = {
                id: _id,
                hasCaret: true,
                icon: "folder-close",
                label: d,
                isExpanded: false,
                childNodes: [],
                nodeData: rootNode
            }
            _id++;
            return node;
       });
        rootNode.childNodes = childNodes;
        setId(_id);
        setNodes([...nodes]);
    });
  }
  
  const getFullPath = (node: ITreeNode): string => {
      let path = node.label.toString();
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
    setSelected(getFullPath(nodeData));
  };

  const handleNodeCollapse = (nodeData: ITreeNode) => {
      nodeData.isExpanded = false;
      setNodes([...nodes]);
  };

  const handleNodeExpand = (nodeData: ITreeNode) => {
      nodeData.isExpanded = true;
      updateNodes(getFullPath(nodeData), nodeData, nodes, id);
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
      <div style={{height, overflowY: 'scroll', width}} className='bp3-table-container'>
        <Tree
            contents={nodes}
            onNodeClick={handleNodeClick}
            onNodeCollapse={handleNodeCollapse}
            onNodeExpand={handleNodeExpand}
            className={Classes.ELEVATION_0}
        />
        </div>
  );
}



