import React from 'react';
import logo from './logo.svg';
import './App.css';
import { Classes, Icon, Intent, ITreeNode, Position, Tooltip, Tree } from "@blueprintjs/core";
import { FolderPicker } from './FolderPicker';
import { MainNavBar } from './MainNavBar';

export interface ITreeState {
  nodes: ITreeNode[];
  homeDir: string;
}


const App: React.FC<{}> = () => {
  return (
    <div className="App">
      <MainNavBar/>
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <FolderPicker nodes={INITIAL_STATE} homeDir=""></FolderPicker>
        <p>
          Edit <code>src/App.tsx</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
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

export default App;

