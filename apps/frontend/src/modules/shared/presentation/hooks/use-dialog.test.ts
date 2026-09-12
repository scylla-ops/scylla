import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useDialog } from './use-dialog';

describe('useDialog', () => {
  it('starts closed with no data', () => {
    const { result } = renderHook(() => useDialog<string>());
    expect(result.current.isOpen).toBe(false);
    expect(result.current.data).toBeNull();
  });

  it('open() with data opens the dialog and stores it', () => {
    const { result } = renderHook(() => useDialog<{ id: string }>());
    act(() => result.current.open({ id: 'agent-1' }));
    expect(result.current.isOpen).toBe(true);
    expect(result.current.data).toEqual({ id: 'agent-1' });
  });

  it('close() closes the dialog and clears the data', () => {
    const { result } = renderHook(() => useDialog<string>());
    act(() => result.current.open('some-id'));
    act(() => result.current.close());
    expect(result.current.isOpen).toBe(false);
    expect(result.current.data).toBeNull();
  });

  it('the void-typed form opens with no argument', () => {
    const { result } = renderHook(() => useDialog());
    act(() => result.current.open());
    expect(result.current.isOpen).toBe(true);
    expect(result.current.data).toBeNull();
  });

  it('opening again with new data replaces the previous data', () => {
    const { result } = renderHook(() => useDialog<string>());
    act(() => result.current.open('first'));
    act(() => result.current.open('second'));
    expect(result.current.data).toBe('second');
  });
});
