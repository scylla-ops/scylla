import { useCallback, useState } from 'react';

export interface DialogState<T = void> {
  isOpen: boolean;
  data: T | null;
  open: [T] extends [void] ? () => void : (data: T) => void;
  close: () => void;
}

export function useDialog<T = void>(): DialogState<T> {
  const [isOpen, setOpen] = useState(false);
  const [data, setData] = useState<T | null>(null);

  const open = useCallback((d?: T) => {
    setData((d ?? null) as T | null);
    setOpen(true);
  }, []);

  const close = useCallback(() => {
    setOpen(false);
    setData(null);
  }, []);

  return { isOpen, data, open: open as DialogState<T>['open'], close };
}
