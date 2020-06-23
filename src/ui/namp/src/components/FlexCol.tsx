import React from 'react';

export const FlexCol: React.FC<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>> = (props) => {
    return (
        <div {...props} style={{...props.style, display: 'flex', flex: 1, flexDirection: 'column' }}>
            {props.children}
        </div>
    )
}