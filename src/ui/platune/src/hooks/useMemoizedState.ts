import { useMemo, useState } from 'react';

export const useMemoizedState = <T extends unknown>(memoVal: T): [T, React.Dispatch<React.SetStateAction<T>>] => {
  const [val, setVal] = useState(memoVal);
  const memoized = useMemo(() => ({ val, setVal }), [val]);
  return [memoized.val, memoized.setVal];
};
