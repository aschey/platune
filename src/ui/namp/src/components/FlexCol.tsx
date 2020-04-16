import React from 'react';

export const FlexCol: React.FC<{children: React.ReactNode, style: React.CSSProperties}> = ({children, style}) => {
    return (
    <div style={{...style, display: 'flex', flex: 1, flexDirection: 'column'}}>
        {children}
    </div>
    )
}