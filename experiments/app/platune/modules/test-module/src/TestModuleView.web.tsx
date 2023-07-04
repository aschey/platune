import * as React from 'react';

import { TestModuleViewProps } from './TestModule.types';

export default function TestModuleView(props: TestModuleViewProps) {
  return (
    <div>
      <span>{props.name}</span>
    </div>
  );
}
