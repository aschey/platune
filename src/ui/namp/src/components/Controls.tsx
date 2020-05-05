import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Text, Label, ProgressBar, Intent, Button, Icon } from '@blueprintjs/core';
import { FlexRow } from './FlexRow';
import { FlexCol } from './FlexCol';

export const Controls: React.FC<{}> = () => {
    return (
        <FlexRow style={{height: 50, alignItems: 'center'}}>
            <FlexCol style={{alignItems: 'center'}}>
                <FlexRow style={{alignItems: 'center'}}>
                    <Button intent={Intent.PRIMARY} outlined icon='fast-backward' style={{borderRadius: '50%', width: 30, height: 30}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.WARNING} outlined icon='pause' style={{borderRadius: '50%', width: 35, height: 35}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.SUCCESS} outlined icon='play' style={{borderRadius: '50%', width: 40, height: 40}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.DANGER} outlined icon='stop' style={{borderRadius: '50%', width: 35, height: 35}}/>
                    <div style={{width: 5}}/>
                    <Button intent={Intent.PRIMARY} outlined icon='fast-forward' style={{borderRadius: '50%', width: 30, height: 30}}/>
                </FlexRow>
            </FlexCol>
        </FlexRow>
    );
    
}