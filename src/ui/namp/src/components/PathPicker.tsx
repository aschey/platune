import React, { useState, useEffect, Children } from 'react';
import { Alert, Intent, IDialogProps, TextArea, Button, Text, Classes, Tooltip } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { FlexRow } from './FlexRow';

interface PathPickerProps {
    width: number,
    height: number,
    buttonHeight: number,
    originalPath: string,
    setOriginalPath: (originalPath: string) => void,
    path: string,
    setPath: (path: string) => void
}

export const PathPicker: React.FC<PathPickerProps> = ({width, height, buttonHeight, originalPath, setOriginalPath, path, setPath}) => {
    //const [selected, setSelected] = useState<string>('');

    useEffect(() => {
        setOriginalPath('/home/aschey');
        setPath('/home/aschey');
    }, []);
    const sepWidth = 10;
    const panelWidth = (width - sepWidth) / 2;
    return (
        <FlexRow style={{alignItems: 'top', alignSelf: 'center', width, height}}>
            <div style={{width: panelWidth}}>
            <Text ellipsize className={Classes.INPUT}>{path}</Text>
                <div style={{height: 5}}/>
                <FlexRow>
                    <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' style={{height: buttonHeight}} />
                    <div style={{margin:5}}/>
                    <Button intent={Intent.WARNING} icon='undo' text='Revert' style={{height: buttonHeight}}/>
                </FlexRow>
            </div>
            <div style={{width: sepWidth}}/>
            <div style={{width: panelWidth, height: height}}>
                <FolderPicker setSelected={setPath}/>
            </div>
            </FlexRow>
        
    )
}
