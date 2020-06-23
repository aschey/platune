import React from 'react';

export const FlexRow: React.FC<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>> = (props) => {
    return (
        <div {...props} style={{...props.style, display: 'flex', flex: 1, flexDirection: 'row'}}>
            {props.children}
        </div>
    )
}