import React, { useState, useEffect } from 'react';
import { getJson } from '../fetchUtil';
import { FlexRow } from './FlexRow';
import { Classes, Text, MenuItem, Position, Button } from '@blueprintjs/core';
import { Suggest, IItemRendererProps, Select } from '@blueprintjs/select';
import { NtfsMapping } from '../models/ntfsMapping';

const DriveSelect = Select.ofType<string>();

interface PathMappingProps {
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
    originalMappings: NtfsMapping[],
    setOriginalMappings: (mappings: NtfsMapping[]) => void,
}

interface RowProps {
    path: NtfsMapping,
    drivesUsed: string[],
    mappings: NtfsMapping[],
    setMappings: (mappings: NtfsMapping[]) => void,
}

const Row: React.FC<RowProps> = ({path, drivesUsed, mappings, setMappings}) => {
    const [selectedRow, setSelectedRow] = useState<string>(path.drive);
    const onItemSelect = (drive: string) => {
        var mapping = mappings.filter(m => m.dir === path.dir)[0];
        mapping.drive = drive;
        setMappings([...mappings]);
        setSelectedRow(drive);
    }  
    return (
        <FlexRow>
            <Text ellipsize className={Classes.INPUT}>{path.dir}</Text>
            <DriveSelect 
                filterable={false}
                items={Array.from(Array(24).keys()).map(i => `${String.fromCharCode(i + 67)}:`)} 
                itemRenderer={(item: string, { handleClick }) => <MenuItem key={item} disabled={drivesUsed.includes(item) && selectedRow !== item} onClick={handleClick} text={item}/>}
                onItemSelect={onItemSelect}
                popoverProps={{minimal: true, popoverClassName:'small'}}>
                <Button text={selectedRow} rightIcon='caret-down' />
            </DriveSelect>
        </FlexRow>
    );
}

export const PathMapping: React.FC<PathMappingProps> = ({mappings, setMappings, originalMappings, setOriginalMappings}) => {
    const choices = Array.from(Array(24).keys()).map(i => `${String.fromCharCode(i + 67)}:`);
    const [drivesUsed, setDrivesUsed] = useState<string[]>([]);

    useEffect(() => {
        getJson<{dir: string, drive: string}[]>('/getNtfsMounts').then(folders => {
            setOriginalMappings([...folders]);
            setMappings([...folders]);
            setDrivesUsed(folders.map(f => f.drive));
        });
    }, []);

    useEffect(() => {
        setDrivesUsed([...mappings.map(m => m.drive)]);
    }, [mappings]);
    
    return (
        <>
            {mappings.map(r => <Row key={r.dir} path={r} drivesUsed={drivesUsed} mappings={mappings} setMappings={setMappings}/>)}
        </>
    );
}