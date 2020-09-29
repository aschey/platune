import { Tab, TabId, Tabs } from '@blueprintjs/core';
import React, { useEffect, useState } from 'react';
import { NtfsMapping } from '../models/ntfsMapping';
import { Dialog } from './Dialog';
import { DirtyCheck } from './DirtyCheck';
import { FolderView } from './FolderView';
import { MultilineText } from './MultilineText';
import { PathMapping } from './PathMapping';
import { PathPicker } from './PathPicker';

export const Settings: React.FC<{
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}> = ({ isOpen, setIsOpen }) => {
  const [closePending, setClosePending] = useState<boolean>(false);
  const [rows, setRows] = useState<Array<string>>([]);
  const [originalRows, setOriginalRows] = useState<Array<string>>([]);
  const [selectedTab, setSelectedTab] = useState<TabId>('f');
  const [chosenTab, setChosenTab] = useState<TabId>('f');
  const [canCloseFolders, setCanCloseFolders] = useState<boolean>(true);
  const [canCloseDbPath, setCanCloseDbPath] = useState<boolean>(true);
  const [alertOpen, setAlertOpen] = useState<boolean>(false);

  const [originalPath, setOriginalPath] = useState<string>('');
  const [path, setPath] = useState<string>('');

  const [originalMappings, setOriginalMappings] = useState<NtfsMapping[]>([]);
  const [mappings, setMappings] = useState<NtfsMapping[]>([]);
  const [canCloseMappings, setCanCloseMappings] = useState<boolean>(true);

  const mapping: Record<TabId, boolean> = {
    f: canCloseFolders,
    t: canCloseDbPath,
    m: canCloseMappings,
  };

  useEffect(() => {
    if (!alertOpen) {
      setClosePending(false);
    }
  }, [alertOpen, setClosePending]);

  const onTabChange = (newTab: TabId) => {
    setChosenTab(newTab);
    if (mapping[selectedTab]) {
      setSelectedTab(newTab);
    } else {
      setAlertOpen(true);
    }
  };

  const onClose = () => {
    if (mapping[selectedTab]) {
      setIsOpen(false);
    } else {
      setClosePending(true);
      setAlertOpen(true);
    }
  };

  const onAlertConfirm = () => {
    if (closePending) {
      setIsOpen(false);
    } else {
      setSelectedTab(chosenTab);
    }
  };

  const height = Math.round(window.innerHeight * 0.66);
  const width = Math.round(window.innerWidth * 0.8);
  const innerWidth = Math.round((width - 260) * 0.9);
  const headerAndMargin = 60;
  const tabHeight = height - headerAndMargin;
  const buttonHeight = 30;
  const buttonPanelHeight = 50;
  const tabHeightNoButtons = tabHeight - buttonPanelHeight;
  const dividerWidth = 10;
  const panelWidth = (innerWidth - dividerWidth) * 0.5;

  const configureFolders = (
    <DirtyCheck
      originalVal={originalRows}
      newVal={rows}
      alertOpen={alertOpen}
      setAlertOpen={setAlertOpen}
      canClose={canCloseFolders}
      setCanClose={setCanCloseFolders}
      onAlertConfirm={onAlertConfirm}
    >
      <FolderView
        width={innerWidth}
        height={tabHeight}
        panelWidth={panelWidth}
        dividerWidth={dividerWidth}
        buttonHeight={buttonHeight}
        buttonPanelHeight={buttonPanelHeight}
        rows={rows}
        setRows={setRows}
        setOriginalRows={setOriginalRows}
      />
    </DirtyCheck>
  );

  const chooseDatabase = (
    <DirtyCheck
      originalVal={originalPath}
      newVal={path}
      alertOpen={alertOpen}
      setAlertOpen={setAlertOpen}
      canClose={canCloseDbPath}
      setCanClose={setCanCloseDbPath}
      onAlertConfirm={onAlertConfirm}
    >
      <PathPicker
        width={innerWidth}
        panelWidth={panelWidth}
        dividerWidth={dividerWidth}
        buttonHeight={buttonHeight}
        marginBottom={buttonPanelHeight}
        height={tabHeightNoButtons}
        setOriginalPath={setOriginalPath}
        path={path}
        setPath={setPath}
      />
    </DirtyCheck>
  );

  const pathMappings = (
    <DirtyCheck
      originalVal={originalMappings}
      newVal={mappings}
      alertOpen={alertOpen}
      setAlertOpen={setAlertOpen}
      canClose={canCloseMappings}
      setCanClose={setCanCloseMappings}
      onAlertConfirm={onAlertConfirm}
    >
      <PathMapping
        width={innerWidth}
        height={tabHeightNoButtons}
        buttonHeight={buttonHeight}
        panelWidth={panelWidth}
        mappings={mappings}
        setMappings={setMappings}
        setOriginalMappings={setOriginalMappings}
      />
    </DirtyCheck>
  );
  return (
    <Dialog
      style={{ width, height }}
      icon='cog'
      title='Settings'
      isOpen={isOpen}
      onClose={onClose}
      autoFocus
      enforceFocus
      usePortal
    >
      <div style={{ paddingLeft: 10, height }}>
        <Tabs vertical selectedTabId={selectedTab} onChange={onTabChange} renderActiveTabPanelOnly>
          <Tab
            id='f'
            title={<MultilineText maxWidth={200} icon='folder-open' text='Import Folders' />}
            panel={configureFolders}
          />
          <Tab
            id='t'
            title={<MultilineText maxWidth={200} icon='database' text='Choose Database Path' />}
            panel={chooseDatabase}
          />
          <Tab
            id='m'
            title={<MultilineText maxWidth={200} icon='arrows-horizontal' text='Path Mappings' />}
            panel={pathMappings}
          />
          <Tab id='b' title={<MultilineText maxWidth={200} icon='updated' text='Backup and Restore' />} />
        </Tabs>
      </div>
    </Dialog>
  );
};
