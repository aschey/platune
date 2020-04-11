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
    getJson('/isWindows').then(async isWindows =>
        { 
            let _delim = isWindows ? '\\' : '/';
            let dirsResponse = await getJson<{dirs: Array<string>}>('/dirsInit');
            let _id = id;
            setDelim(_delim);
            let _nodes = dirsResponse.dirs.map((d, i): ITreeNode => {
                let node: ITreeNode = {
                    id: _id,
                    hasCaret: true,
                    icon: 'folder-close',
                    label: d,
                    isExpanded: false,
                    childNodes: [],
                    nodeData: undefined
                }
                _id++;
                return node;
            });
            for (let node of _nodes) {
                _id = await updateNodes(getFullPath(node), node, _nodes, _id + 1);
            }
            
            
        });
    
  }, []);

  const updateNodes = async (path: string, rootNode: ITreeNode, nodes: ITreeNode[], startId: number): Promise<number> => {
    const dirsResponse = await getJson<{dirs: Array<string>}>(`/dirs?dir=${path}`);
    let _id = startId;
    const shouldExpand = dirsResponse.dirs.length > 1;
    const childNodes = dirsResponse.dirs.map((d, i): ITreeNode => {
        let node: ITreeNode = {
            id: _id,
            hasCaret: true,
            icon: 'folder-close',
            label: d,
            isExpanded: shouldExpand,
            childNodes: [],
            nodeData: rootNode
        }
        _id++;
        return node;
    });
    rootNode.childNodes = childNodes;
    setId(_id);
    setNodes([...nodes]);
    return _id;
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



