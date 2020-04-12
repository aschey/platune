import React, { useState, useEffect } from 'react';
import { Dialog as BlueprintDialog, IDialogProps as BlueprintDialogProps } from "@blueprintjs/core";

interface DialogProps extends BlueprintDialogProps {
    children: React.ReactNode
}

export const Dialog: React.FC<DialogProps> = (props: DialogProps) => {
    return <BlueprintDialog style={{boxShadow: 'none', paddingBottom: 0, ...props.style}} className='bp3-dark' {...props}>
       {props.children}
    </BlueprintDialog>
}