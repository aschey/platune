import React, { useState, useEffect, Children } from 'react';
import { Alert, Intent, IDialogProps, TextArea, Button, Text, Classes, Tooltip, Colors } from '@blueprintjs/core';
import { FolderPicker } from './FolderPicker';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';
import { getJson, putJson } from '../fetchUtil';
import { Dir } from '../models/dir';
import { toastSuccess } from '../appToaster';

interface PathPickerProps {
    width: number,
    height: number,
    marginBottom: number,
    buttonHeight: number,
    setOriginalPath: (originalPath: string) => void,
    path: string,
    setPath: (path: string) => void
}

export const PathPicker: React.FC<PathPickerProps> = ({width, height, buttonHeight, setOriginalPath, path, setPath, marginBottom}) => {
    const [databaseFound, setDatabaseFound] = useState<boolean>(false);
    useEffect(() => {
        getJson<{name: string}>('/getDbPath').then(res => {
            setOriginalPath(res.name);
            setPath(res.name);
        });
        
    }, [setOriginalPath, setPath]);

    useEffect(() => {
        if (path === '') {
            return;
        }
        getJson<{dirs: Dir[]}>(`/dirs?dir=${path}`).then(res => {
            setDatabaseFound(res.dirs.some(d => d.isFile && d.name.endsWith('namp.db')));
        })
    }, [path, databaseFound, setDatabaseFound]);

    const onSaveClick = async () => {
        await putJson<{}>('/updateDbPath', { dir: path});
        setOriginalPath(path);
        toastSuccess();
    }

    const dbFound = 
        <div style={{color: Colors.GREEN2, paddingTop: 5, paddingLeft: 5}}>
            <Text>* Existing database found</Text>
        </div>;
    const dbNotFound = 
        <div style={{color: Colors.ORANGE2, paddingTop: 5, paddingLeft: 5}}>
            <Text>* Existing database not found</Text>
        </div>;

    const sepWidth = 10;
    const panelWidth = (width - sepWidth) / 2;
    return (
        <FlexRow style={{alignItems: 'top', alignSelf: 'center', width, height: height - marginBottom}}>
            <div style={{width: panelWidth}} className={`${Classes.getClassNamespace()}-table-container`}>
                <div style={{margin: 5}}>
                <Text ellipsize className={Classes.INPUT}>{path}</Text>
                { databaseFound ? dbFound : dbNotFound }
            </div>
            <div style={{height: 5}}/>
                <FlexRow style={{margin: 5}}>
                    <Button intent={Intent.SUCCESS} icon='floppy-disk' text='Save' style={{height: buttonHeight}} onClick={onSaveClick}/>
                    <div style={{margin:5}}/>
                    <Button intent={Intent.WARNING} icon='undo' text='Revert' style={{height: buttonHeight}}/>
                </FlexRow>
            </div>
            <div style={{width: sepWidth}}/>
            <div style={{width: panelWidth, height: height - marginBottom}}>
                <FolderPicker setSelected={setPath}/>
            </div>
        </FlexRow>
    )
}
