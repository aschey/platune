import { requireNativeViewManager } from 'expo-modules-core';
import * as React from 'react';

import { TestModuleViewProps } from './TestModule.types';

const NativeView: React.ComponentType<TestModuleViewProps> =
  requireNativeViewManager('TestModule');

export default function TestModuleView(props: TestModuleViewProps) {
  return <NativeView {...props} />;
}
