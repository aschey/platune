import { NativeModulesProxy, EventEmitter, Subscription } from 'expo-modules-core';

// Import the native module. On web, it will be resolved to TestModule.web.ts
// and on native platforms to TestModule.ts
import TestModule from './src/TestModule';
import TestModuleView from './src/TestModuleView';
import { ChangeEventPayload, TestModuleViewProps } from './src/TestModule.types';

// Get the native constant value.
export const PI = TestModule.PI;

export function hello(): string {
  return TestModule.hello();
}

export async function setValueAsync(value: string) {
  return await TestModule.setValueAsync(value);
}

const emitter = new EventEmitter(TestModule ?? NativeModulesProxy.TestModule);

export function addChangeListener(listener: (event: ChangeEventPayload) => void): Subscription {
  return emitter.addListener<ChangeEventPayload>('onChange', listener);
}

export { TestModuleView, TestModuleViewProps, ChangeEventPayload };
