import React, { useState, useEffect, useCallback } from 'react';
import { Table, Column, Cell } from '@blueprintjs/table';
import { Button, ITreeNode, Tooltip, Position, Icon, Classes, Intent, Toaster, Toast, ButtonGroup, Divider, Alert } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { getJson, putJson } from '../fetchUtil';
import { SelectedFolders } from './SelectedFolders';
import { FlexCol } from './FlexCol';
import { FlexRow } from './FlexRow';

const AppToaster = Toaster.create({
    position: Position.TOP
});

interface FolderViewProps {
    width: number;
    height: number;
    buttonHeight: number;
    buttonPanelHeight: number;
    rows: string[];
    setRows: (rows: string[]) => void;
    setOriginalRows: (rows: string[]) => void;
}


export const FolderView: React.FC<FolderViewProps> = ({width, height, buttonHeight, buttonPanelHeight, rows, setRows, setOriginalRows}) => {
    const [selected, setSelected] = useState<string>('');
    const [errorText, setErrorText] = useState<string>('');

    const refreshFolders = useCallback(() => getJson<Array<string>>('/configuredFolders').then(folders => {
        setRows([...folders]);
        setOriginalRows([...folders]);
    }), [setRows, setOriginalRows]);
    
    useEffect(() => {
      refreshFolders();
      return () => setRows([]);
    }, [refreshFolders, setRows]);

    const cellRenderer = (rowIndex: number) => {
        return <Cell>{rows[rowIndex]}</Cell>
    };

    const addFolderClick = () => {
        setRows([...rows, selected]);
    }

    const saveFoldersClick = async () => {
        try {
            await putJson<void>('/updateFolders', {folders: rows});
            setOriginalRows([...rows]);
            AppToaster.show({message: 'Success', intent: Intent.SUCCESS, icon: 'tick-circle', timeout: 1000});
        }
        catch (e) {
            setErrorText(e.message);
        }
        
    }

    const revertClick = () => {
        refreshFolders();
    }

    const spacerWidth = 10;
    const panelWidth = (width - spacerWidth) / 2;
    return (
        <>
            <FlexRow style={{alignItems: 'top', alignSelf: 'center', width, height}}>
                <FlexCol style={{width: panelWidth}}>
                    <div style={{height: height-buttonPanelHeight}}>
                        <SelectedFolders rows={rows} setRows={setRows} width={panelWidth}/>
                    </div>
                    <FlexRow style={{alignItems: 'center'}}>
                        <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' style={{height: buttonHeight}} onClick={saveFoldersClick}/>
                        <div style={{margin:spacerWidth}}/>
                        <Button intent={Intent.WARNING} icon='undo' text='Revert' style={{height: buttonHeight}} onClick={revertClick}/>
                    </FlexRow>
                </FlexCol>
                <div style={{width: spacerWidth}}/>
                <FlexCol style={{width: panelWidth}}>
                    <div style={{height: height-buttonPanelHeight}}>
                        <FolderPicker setSelected={setSelected}/>
                    </div>
                    <FlexRow style={{ alignItems: 'center' }}>
                        <Button intent={Intent.PRIMARY} onClick={addFolderClick} icon='add' text='Add' style={{height: buttonHeight}}/>
                    </FlexRow>
                </FlexCol>
            </FlexRow>
            <Alert intent={Intent.DANGER} isOpen={errorText.length > 0} className={`bp3-dark`} onClose={() => setErrorText('')}>
                {errorText}
            </Alert>
        </>
    )
}
