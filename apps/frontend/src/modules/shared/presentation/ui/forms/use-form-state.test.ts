import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useFormState } from './ScyllaForm';
import { FormItemType, type FormItem } from '@shared/presentation/structs/scylla-form.struct.ts';

type Ids = 'username' | 'bio' | 'email';

const items: readonly FormItem<Ids>[] = [
  { id: 'username', type: FormItemType.Input, inputType: 'text', label: 'Username' },
  { id: 'bio', type: FormItemType.Input, inputType: 'text', label: 'Bio', optional: true },
  {
    id: 'email',
    type: FormItemType.Input,
    inputType: 'email',
    label: 'Email',
    pattern: '^[^@]+@[^@]+\\.[^@]+$',
  },
];

describe('useFormState', () => {
  it('seeds values from each item\'s defaultValue, or empty string when none', () => {
    const withDefaults: readonly FormItem<'a' | 'b'>[] = [
      { id: 'a', type: FormItemType.Input, inputType: 'text', label: 'A', defaultValue: 'hi' },
      { id: 'b', type: FormItemType.Input, inputType: 'text', label: 'B' },
    ];
    const { result } = renderHook(() => useFormState(withDefaults));
    expect(result.current.values).toEqual({ a: 'hi', b: '' });
  });

  it('handleChange updates only the targeted field', () => {
    const { result } = renderHook(() => useFormState(items));
    act(() => result.current.handleChange('username', 'ravenne'));
    expect(result.current.values.username).toBe('ravenne');
    expect(result.current.values.bio).toBe('');
  });

  it('reset restores every field to its default', () => {
    const { result } = renderHook(() => useFormState(items));
    act(() => result.current.handleChange('username', 'ravenne'));
    act(() => result.current.reset());
    expect(result.current.values).toEqual({ username: '', bio: '', email: '' });
  });

  it('is invalid while a required field is empty', () => {
    const { result } = renderHook(() => useFormState(items));
    expect(result.current.isValid).toBe(false);
  });

  it('an optional field being empty does not block validity', () => {
    const onlyOptional: readonly FormItem<'bio'>[] = [
      { id: 'bio', type: FormItemType.Input, inputType: 'text', label: 'Bio', optional: true },
    ];
    const { result } = renderHook(() => useFormState(onlyOptional));
    expect(result.current.isValid).toBe(true);
  });

  it('a required field that is only whitespace still counts as empty', () => {
    const { result } = renderHook(() => useFormState(items));
    act(() => result.current.handleChange('username', '   '));
    act(() => result.current.handleChange('email', 'a@b.co'));
    expect(result.current.isValid).toBe(false);
  });

  it('enforces the item pattern once a value is present', () => {
    const { result } = renderHook(() => useFormState(items));
    act(() => result.current.handleChange('username', 'ravenne'));
    act(() => result.current.handleChange('email', 'not-an-email'));
    expect(result.current.isValid).toBe(false);

    act(() => result.current.handleChange('email', 'ravenne@scylla.dev'));
    expect(result.current.isValid).toBe(true);
  });

  it('is valid once every required field is filled and every pattern matches', () => {
    const { result } = renderHook(() => useFormState(items));
    act(() => result.current.handleChange('username', 'ravenne'));
    act(() => result.current.handleChange('email', 'ravenne@scylla.dev'));
    expect(result.current.isValid).toBe(true);
  });

  it('a pattern on a Select-type item (impossible in the type, but defensive) is never checked', () => {
    // FormSelect has no `pattern`, so this documents that pattern-checking is
    // gated on FormItemType.Input specifically, not "has a pattern field".
    const selectItem: readonly FormItem<'plan'>[] = [
      {
        id: 'plan',
        type: FormItemType.Select,
        label: 'Plan',
        options: [{ label: 'Free', value: 'free' }],
        defaultValue: 'free',
      },
    ];
    const { result } = renderHook(() => useFormState(selectItem));
    expect(result.current.isValid).toBe(true);
  });
});
