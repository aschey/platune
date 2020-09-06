import React from 'react';
import { Dialog } from './Dialog';

interface AddEditTagProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}
export const AddEditTag: React.FC<AddEditTagProps> = ({ isOpen, setIsOpen }) => {
  return (
    <Dialog
      style={{ width: 500, height: 200 }}
      icon='add'
      title='New Tag'
      isOpen={isOpen}
      onClose={() => setIsOpen(false)}
      autoFocus
      enforceFocus
      usePortal
    >
      <div />
    </Dialog>
  );
};
