import { Toaster, Position, Intent } from '@blueprintjs/core';

const appToaster = Toaster.create({
  position: Position.TOP,
});

export const toastSuccess = () => {
  appToaster.show({ message: 'Success', intent: Intent.SUCCESS, icon: 'tick-circle', timeout: 1000 });
};

export const toastMessage = (message: string) => {
  appToaster.show({ message, intent: Intent.PRIMARY, timeout: 1000 });
};
