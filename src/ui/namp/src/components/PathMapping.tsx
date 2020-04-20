import React, { useState, useEffect } from 'react';
import { getJson } from '../fetchUtil';
import { FlexRow } from './FlexRow';
import { Classes, Text, MenuItem, Position, Button } from '@blueprintjs/core';
import { Suggest, IItemRendererProps, Select } from '@blueprintjs/select';
import { NtfsMapping } from '../models/ntfsMapping';
import { FlexCol } from './FlexCol';
import _ from 'lodash';

const DriveSelect = Select.ofType<string>();

interface PathMappingProps {
    width: number, 
    height: number,
    panelWidth: number,
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
    setOriginalMappings: (mappings: NtfsMapping[]) => void,
}

interface RowProps {
    path: NtfsMapping,
    drivesUsed: string[],
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
    width: number
}

const NONE = 'None';

const Row: React.FC<RowProps> = ({path, drivesUsed, mappings, setMappings, width}) => {
    const [selectedRow, setSelectedRow] = useState<string>(path.drive);
    useEffect(() => {
        setSelectedRow(path.drive);
    }, [path]);
    
    let choices = [NONE, ...Array.from(Array(24).keys()).map(i => `${String.fromCharCode(i + 67)}:`)];
    const onItemSelect = (drive: string) => {
        var mapping = mappings.filter(m => m.dir === path.dir)[0];
        mapping.drive = drive;
        setMappings(_.cloneDeep(mappings));
        setSelectedRow(drive);
    }  
    return (
        <FlexRow style={{padding: 5}}>
            <FlexCol style={{minWidth: width - 80, paddingRight: 10}}>
                <Text ellipsize className={Classes.INPUT}>{path.dir}</Text>
            </FlexCol>
            <FlexCol>
                <DriveSelect 
                    filterable={false}
                    items={choices} 
                    itemRenderer={(item: string, { handleClick }) => <MenuItem key={item} disabled={drivesUsed.includes(item) && selectedRow !== item && item !== NONE} onClick={handleClick} text={item}/>}
                    onItemSelect={onItemSelect}
                    popoverProps={{minimal: true, popoverClassName:'small'}}>
                    <Button text={selectedRow} style={{width: 70}} rightIcon='caret-down' />
                </DriveSelect>
            </FlexCol>
        </FlexRow>
    );
}

export const PathMapping: React.FC<PathMappingProps> = ({mappings, setMappings, setOriginalMappings, width, height, panelWidth}) => {
    const [drivesUsed, setDrivesUsed] = useState<string[]>([]);

    useEffect(() => {
        getJson<{dir: string, drive: string}[]>('/getNtfsMounts').then(folders => {
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
    return (
        <div className={'bp3-table-container'} style={{height}}>
            <div style={{width: panelWidth, paddingTop: 10}}>
                {mappings.map(r => <Row key={r.dir} width={panelWidth} path={r} drivesUsed={drivesUsed} mappings={mappings} setMappings={setMappings}/>)}
            </div>
        </div>
    );
}