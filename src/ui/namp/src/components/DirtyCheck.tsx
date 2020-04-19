import React, { useState, useEffect, Children, useCallback } from 'react';
import { Alert, Intent, IDialogProps } from '@blueprintjs/core';

interface DirtyCheckProps<T> {
    children: React.ReactElement,
    alertOpen: boolean,
    //checkEqual: (left: T, right: T) => boolean,
    originalVal: T,
    newVal: T,
    canClose: boolean,
    setCanClose: (canClose: boolean) => void,
    onAlertConfirm: () => void,
    setAlertOpen: (isOpen: boolean) => void,
}

export const DirtyCheck: <T>(props: DirtyCheckProps<T>) => React.ReactElement<DirtyCheckProps<T>> = (props) => {
    const { children, originalVal, newVal, alertOpen, setAlertOpen, setCanClose } = props;
    const propsAlertConfirm = props.onAlertConfirm;

    

    const checkEqual = useCallback((originalVal: any, newVal: any) => {
        const arraysEqual = <T extends any>(a: T[], b: T[]): boolean => {
            if (a === b) return true;
            if (a == null || b == null) return false;
            if (a.length !== b.length) return false;
        
            const sortedA = a.concat().sort();
            const sortedB = b.concat().sort();
            for (var i = 0; i < sortedA.length; ++i) {
                if (!checkEqual(sortedA[i], sortedB[i])) {
                    return false;
                }
            }
            return true;
        };

        const sortObject = (val: {}) => Object.keys(originalVal).sort().reduce((result, key) => {
            result[key] = originalVal[key];
            return result;
        }, {});

        if (Array.isArray(originalVal) && Array.isArray(newVal)) {
            return arraysEqual(originalVal, newVal);
        }
        if (typeof originalVal === 'object' && typeof newVal === 'object') {
            const sortedA = sortObject(originalVal);
            const sortedB = sortObject(newVal);
            return JSON.stringify(sortedA) === JSON.stringify(sortedB);
        }

        return originalVal === newVal;
    }, [originalVal, newVal]);

    useEffect(() => {
        setCanClose(checkEqual(originalVal, newVal));
    }, [originalVal, newVal, checkEqual, setCanClose]);

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
