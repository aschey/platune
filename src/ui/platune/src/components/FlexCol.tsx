import React from 'react';

interface FlexColProps extends React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement> {
  center?: boolean;
}

export const FlexCol: React.FC<FlexColProps> = props => {
  const style: React.CSSProperties = { ...props.style, display: 'flex', flex: 1, flexDirection: 'column' };
  if (props.center !== false && !props.style?.alignItems) {
    style.alignItems = 'center';
  }
  if (props.center !== false && !props.style?.alignContent) {
    style.alignContent = 'center';
  }
  return (
    <div {...props} style={style}>
      {props.children}
    </div>
  );
};
