import React from 'react';

export const FlexCol: React.FC<React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>> = (props) => {
    return (
        <div style={{...props.style, display: 'flex', flex: 1, flexDirection: 'row'}} {...props}>
            {props.children}
        </div>
    )
}