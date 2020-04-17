import React, { useState, useEffect, Children } from 'react';
import { Alert, Intent, IDialogProps } from '@blueprintjs/core';

interface DirtyCheckProps<T> {
    children: React.ReactElement,
    alertOpen: boolean,
    checkEqual: (left: T, right: T) => boolean,
    originalVal: T,
    newVal: T,
    canClose: boolean,
    setCanClose: (canClose: boolean) => void,
    onAlertConfirm: () => void,
    setAlertOpen: (isOpen: boolean) => void,
}

export const DirtyCheckDialog: <T>(props: DirtyCheckProps<T>) => React.ReactElement<DirtyCheckProps<T>> = (props) => {
    const { children, checkEqual, originalVal, newVal, alertOpen, setAlertOpen, setCanClose } = props;
    const propsAlertConfirm = props.onAlertConfirm;

    useEffect(() => {
        setCanClose(checkEqual(originalVal, newVal));
    }, [originalVal, newVal, checkEqual]);

    const onAlertConfirm = () => {
        setAlertOpen(false);
        setCanClose(true);
        propsAlertConfirm();
    }

    const onAlertCancel = () => {
        setAlertOpen(false);
        setCanClose(false);
    }

    return (
        <>
        {children}
        <Alert intent={Intent.DANGER} isOpen={alertOpen} className={`bp3-dark`} onConfirm={onAlertConfirm} confirmButtonText='Discard' cancelButtonText='Cancel' onCancel={onAlertCancel}>
            You have unsaved changes
        </Alert>
        </>
        
    );
}
