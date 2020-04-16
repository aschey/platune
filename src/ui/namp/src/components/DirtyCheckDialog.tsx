import React, { useState, useEffect } from 'react';
import { Alert, Intent, IDialogProps } from '@blueprintjs/core';
import { Dialog } from './Dialog';

interface DirtyCheckDialogProps<T> {
    children: React.ReactNode,
    checkEqual: (left: T, right: T) => boolean,
    originalVal: T,
    newVal: T,
    isOpen: boolean,
    setIsOpen: (isOpen: boolean) => void,
    style: React.CSSProperties
}

export const DirtyCheckDialog: <T>(props: DirtyCheckDialogProps<T>) => React.ReactElement<DirtyCheckDialogProps<T>> = (props) => {
    const { children, checkEqual, originalVal, newVal, isOpen, setIsOpen, style } = props;
    const [canClose, setCanClose] = useState<boolean>(true);
    const [alertOpen, setAlertOpen] = useState<boolean>(false);

    useEffect(() => {
        setCanClose(checkEqual(originalVal, newVal));
    }, [originalVal, newVal, checkEqual])

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
            style={style}
            icon='folder-open' 
            title='Configure Folders' 
            isOpen={isOpen} 
            onClose={onClose}
            autoFocus={true}
            enforceFocus={true}
            usePortal={true}>
            {children}
        </Dialog>
        <Alert intent={Intent.DANGER} isOpen={alertOpen} className={`bp3-dark`} onConfirm={onAlertConfirm} confirmButtonText='Discard' cancelButtonText='Cancel' onCancel={onAlertCancel}>
            You have unsaved changes
        </Alert>
    </>)
}