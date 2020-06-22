import React from 'react';

export const FlexRow: React.FC<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>> = (props) => {
    return (
        <div style={{...props.style, display: 'flex', flex: 1, flexDirection: 'column'}} {...props}>
            {props.children}
        </div>
    )
}