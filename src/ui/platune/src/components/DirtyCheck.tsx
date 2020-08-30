import { Alert, Intent } from '@blueprintjs/core';
import _ from 'lodash';
import React, { useEffect } from 'react';

interface DirtyCheckProps<T> {
  children: React.ReactElement;
  alertOpen: boolean;
  originalVal: T;
  newVal: T;
  canClose: boolean;
  setCanClose: (canClose: boolean) => void;
  onAlertConfirm: () => void;
  setAlertOpen: (isOpen: boolean) => void;
}

export const DirtyCheck: <T>(props: DirtyCheckProps<T>) => React.ReactElement<DirtyCheckProps<T>> = props => {
  const { children, originalVal, newVal, alertOpen, setAlertOpen, setCanClose } = props;
  const propsAlertConfirm = props.onAlertConfirm;

  useEffect(() => {
    setCanClose(_.isEqual(originalVal, newVal));
  }, [originalVal, newVal, setCanClose]);

  const onAlertConfirm = () => {
    setAlertOpen(false);
    setCanClose(true);
    propsAlertConfirm();
  };

  const onAlertCancel = () => {
    setAlertOpen(false);
    setCanClose(false);
  };

  return (
    <>
      {children}
      <Alert
        intent={Intent.DANGER}
        isOpen={alertOpen}
        onConfirm={onAlertConfirm}
        confirmButtonText='Discard'
        cancelButtonText='Cancel'
        onCancel={onAlertCancel}
      >
        You have unsaved changes
      </Alert>
    </>
  );
};
