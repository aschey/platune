import React, { useState, useEffect } from 'react';
import { Alert, Intent, Dialog, IDialogProps } from '@blueprintjs/core';

interface DirtyCheckProps<T> extends IDialogProps {
    children: React.ReactNode,
    checkEqual: <T>(left: T, right: T) => boolean,
    originalVal: T,
    newVal: T
}

export const DirtyCheck: React.FC<DirtyCheckProps<{}>> = <T extends {}>(props: DirtyCheckProps<T>) => {
    const [isOpen, setIsOpen] = useState<boolean>(false);
    const [canClose, setCanClose] = useState<boolean>(true);
    const [alertOpen, setAlertOpen] = useState<boolean>(false);

    useEffect(() => {
        setCanClose(props.checkEqual(props.originalVal, props.newVal));
    }, [props.originalVal, props.newVal])

    const onClose = () => {
        if (canClose) { 
            setIsOpen(false);
        }
        else {
            setAlertOpen(true);
        }
    }

    const onAlertConfirm = () => {
        setAlertOpen(false);
        setCanClose(true);
        setIsOpen(false);
    }

    const onAlertCancel = () => {
        setAlertOpen(false);
        setCanClose(false);
        setIsOpen(true);
    }

    return (
    <>
        <Dialog 
            style={props.style}
            icon='folder-open' 
            title='Configure Folders' 
            isOpen={isOpen} 
            onClose={onClose}
            autoFocus={true}
            enforceFocus={true}
            usePortal={true}>
            {props.children}
        </Dialog>
    
        <Alert intent={Intent.DANGER} isOpen={alertOpen} className={`bp3-dark`} onConfirm={onAlertConfirm} confirmButtonText='Discard' cancelButtonText='Cancel' onCancel={onAlertCancel}>
            'You have unsaved changes'
        </Alert>
    </>)
}