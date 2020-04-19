import React, { useState, useEffect } from 'react';
import { getJson } from '../fetchUtil';
import { FlexRow } from './FlexRow';
import { Classes, Text, MenuItem, Position } from '@blueprintjs/core';
import { Suggest, IItemRendererProps } from '@blueprintjs/select';

const DriveSuggest = Suggest.ofType<string>();

interface PathMappingProps {
    mappings: string[],
    setMappings: (mappings: string[]) => void,
    originalMappings: string[],
    setOriginalMappings: (mappings: string[]) => void,
}

export const PathMapping: React.FC<PathMappingProps> = ({mappings, setMappings, originalMappings, setOriginalMappings}) => {
    //const [rows, setRows] = useState<string[]>([]);
    const [selectedRow, setSelectedRow] = useState<string>('C:');
    useEffect(() => {
        getJson<string[]>('/configuredFolders').then(folders => {
            setOriginalMappings([...folders]);
            setMappings([...folders]);
        });
    }, []);

    const row = (path: string) => 
        <FlexRow key={path}>
            <Text ellipsize className={Classes.INPUT}>{path}</Text>
            <DriveSuggest 
                selectedItem={selectedRow}
                closeOnSelect={true}
                items={Array.from(Array(24).keys()).map(i => `${String.fromCharCode(i + 67)}:`)} 
                inputValueRenderer={(drive: string) => drive} 
                itemRenderer={(item: string, { handleClick }) => <MenuItem key={item} onClick={handleClick} text={item}/>}
                onItemSelect={(drive: string) => {setSelectedRow(drive)}}
                popoverProps={{minimal: true}}/>
        </FlexRow>;
    return (
        <>
            {mappings.map(r => row(r))}
        </>
    );
}