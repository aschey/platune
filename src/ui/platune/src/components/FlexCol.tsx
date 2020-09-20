import React from 'react';

interface FlexColProps extends React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement> {
  center?: boolean;
}

export const FlexCol = React.forwardRef<HTMLDivElement, FlexColProps>((props, ref) => {
  const style: React.CSSProperties = { ...props.style, display: 'flex', flex: 1, flexDirection: 'column' };
  if (props.center !== false && !props.style?.alignItems) {
    style.alignItems = 'center';
  }
  if (props.center !== false && !props.style?.alignContent) {
    style.alignContent = 'center';
  }
  const { center, ...styleProps } = props;
  return (
    <div {...styleProps} style={style} ref={ref}>
      {props.children}
    </div>
  );
});
