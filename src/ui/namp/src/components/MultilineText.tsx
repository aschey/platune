import React from 'react';
import { Icon, IconName } from '@blueprintjs/core';

interface MultilineTextProps {
    icon: IconName,
    maxWidth: number,
    text: string
}

export const MultilineText: React.FC<MultilineTextProps> = ({icon, maxWidth, text}) => {
    return (
        <>
            <span style={{whiteSpace: 'pre', verticalAlign: 'top'}}><Icon icon={icon}/>  </span>
            <span style={{whiteSpace: 'normal', maxWidth: maxWidth, wordWrap: 'break-word', display: 'inline-block'}}>
                {text}
            </span>
        </>
    );
}