import React from 'react';

export const FlexRow: React.FC<{children: React.ReactNode, style?: React.CSSProperties, flex?: number}> = ({children, style, flex}) => {
    return (
    <div style={{...style, display: 'flex', flex: flex ?? 1, flexDirection: 'row'}}>
        {children}
    </div>
    )
}