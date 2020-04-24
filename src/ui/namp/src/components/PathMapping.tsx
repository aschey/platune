import React, { useState, useEffect } from 'react';
import { getJson, putJson } from '../fetchUtil';
import { FlexRow } from './FlexRow';
import { Classes, Text, MenuItem, Position, Button, Intent, EditableText } from '@blueprintjs/core';
import { Suggest, IItemRendererProps, Select } from '@blueprintjs/select';
import { NtfsMapping } from '../models/ntfsMapping';
import { FlexCol } from './FlexCol';
import _ from 'lodash';
import { toastSuccess } from '../appToaster';

const DriveSelect = Select.ofType<string>();

interface PathMappingProps {
    width: number, 
    height: number,
    panelWidth: number,
    buttonHeight: number,
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
    setOriginalMappings: (mappings: NtfsMapping[]) => void,
}

interface RowProps {
    path: NtfsMapping,
    drivesUsed: string[],
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
    width: number,
    isWindows: boolean,
    index: number
}

const NONE = 'None';

const Row: React.FC<RowProps> = ({path, drivesUsed, mappings, setMappings, width, isWindows, index}) => {
    const [selectedRow, setSelectedRow] = useState<string>(path.drive);
    const [newDir, setNewDir] = useState<string>(path.dir);
    useEffect(() => {
        setSelectedRow(path.drive);
    }, [path]);
    
    let choices = [NONE, ...Array.from(Array(24).keys()).map(i => `${String.fromCharCode(i + 67)}:`)];
    const onItemSelect = (drive: string) => {
        var mapping = mappings[index];
        mapping.drive = drive;

        setMappings(_.cloneDeep(mappings));
        setSelectedRow(drive);
    }  

    const onConfirm = (text: string) => {
        var mapping = mappings[index];
        mapping.dir = text;

        setMappings(_.cloneDeep(mappings));
        setNewDir(text);
    }

    const onDelete = () => {
        mappings.splice(index, 1);
        setMappings(_.cloneDeep(mappings));
    }

    const driveSelect = (
        <DriveSelect 
            filterable={false}
            items={choices} 
            itemRenderer={(item: string, { handleClick }) => <MenuItem key={item} disabled={drivesUsed.includes(item) && selectedRow !== item && item !== NONE} onClick={handleClick} text={item}/>}
            onItemSelect={onItemSelect}
            popoverProps={{minimal: true, popoverClassName:'small'}}>
            <Button text={selectedRow} style={{width: 70}} rightIcon='caret-down' />
        </DriveSelect>
    );

    const unixRow = (
        <FlexRow style={{padding: 5}}>
            <FlexCol style={{minWidth: width - 80, paddingRight: 10}}>
                <Text ellipsize className={Classes.INPUT}>{path.dir}</Text>
            </FlexCol>
            <FlexCol>
                {driveSelect}
            </FlexCol>
        </FlexRow>
    );

    const windowsRow = (
        <FlexRow style={{padding: 5}}>
            <FlexCol style={{maxWidth: 80}}>
                {driveSelect}
            </FlexCol>
            <FlexCol style={{paddingRight: 10}}>
                <EditableText className={Classes.INPUT} value={newDir} onChange={setNewDir} onConfirm={onConfirm}/>
            </FlexCol>
            <Button intent={Intent.DANGER} icon='delete' onClick={onDelete} />
        </FlexRow>
    )

    return isWindows ? windowsRow : unixRow;
}

export const PathMapping: React.FC<PathMappingProps> = ({mappings, setMappings, setOriginalMappings, width, height, panelWidth, buttonHeight}) => {
    const [drivesUsed, setDrivesUsed] = useState<string[]>([]);
    const [isWindows, setIsWindows] = useState<boolean>(false);

    useEffect(() => {
        getJson<boolean>('/isWindows').then(async isWindows => {
            setIsWindows(isWindows);
            let folders = await getJson<NtfsMapping[]>('/getNtfsMounts');
            if (isWindows && folders.length === 0) {
                folders.push({dir: '', drive: ''});
            }
            folders.forEach(f => {
                if (f.drive === '') {
                    f.drive = NONE;
                }
            });
            setOriginalMappings(_.cloneDeep(folders));
            setMappings(folders);
            setDrivesUsed(folders.map(f => f.drive));
        });
    }, [setOriginalMappings, setMappings, setDrivesUsed]);

    useEffect(() => {
        setDrivesUsed([...mappings.map(m => m.drive)]);
    }, [mappings]);

    const onSaveClick = async () => {
        await putJson<{}>("/updatePathMappings", mappings.map((m): NtfsMapping => ({dir: m.dir, drive: m.drive === NONE ? '' : m.drive})));
        toastSuccess();
        setOriginalMappings(_.cloneDeep(mappings));
    }

    const onRevertClick = () => {
        getJson<NtfsMapping[]>('/getNtfsMounts').then(folders => {
            folders.forEach(f => {
                if (f.drive === '') {
                    f.drive = NONE;
                }
            });
            setOriginalMappings(_.cloneDeep(folders));
            setMappings(folders);
            setDrivesUsed(folders.map(f => f.drive));
        });
    }

    const onAddClick = () => {
        setMappings([..._.cloneDeep(mappings), { dir: '', drive: 'None'}]);
    }

    const addButton = 
        <>
            <Button intent={Intent.PRIMARY} icon='plus' text='Add' style={{height: buttonHeight}} onClick={onAddClick}/>
            <div style={{margin:5}}/>
        </>; 
    const noMappings = !isWindows && mappings.length === 0;
    return (
        <div className={'bp3-table-container'} style={{height}}>
            <div style={{width: panelWidth, paddingTop: 10}}>
                {noMappings
                    ? <div style={{paddingLeft: 5}}><Text>No mappings available</Text></div>
                    : mappings.map((r, i) => <Row key={i} index={i} isWindows={isWindows} width={panelWidth} path={r} drivesUsed={drivesUsed} mappings={mappings} setMappings={setMappings}/>)}
            </div>
            <div style={{height: 5}}/>
            <FlexRow style={{margin: 5, marginLeft: 5}}>
                { isWindows ? addButton : null }
                <Button intent={Intent.SUCCESS} disabled={noMappings} icon='floppy-disk' text='Save' style={{height: buttonHeight}} onClick={onSaveClick}/>
                <div style={{margin:5}}/>
                <Button intent={Intent.WARNING} disabled={noMappings} icon='undo' text='Revert' style={{height: buttonHeight}} onClick={onRevertClick}/>
            </FlexRow>
        </div>
    );
}