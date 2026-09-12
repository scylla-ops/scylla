import { describe, it, expect } from 'vitest';
import { cn } from './cn';

describe('cn', () => {
  it('joins plain class strings', () => {
    expect(cn('a', 'b')).toBe('a b');
  });

  it('drops falsy inputs (undefined, null, false, empty string)', () => {
    expect(cn('a', undefined, null, false, '', 'b')).toBe('a b');
  });

  it('flattens arrays and objects the way clsx does', () => {
    expect(cn(['a', 'b'], { c: true, d: false })).toBe('a b c');
  });

  it('resolves conflicting Tailwind utilities to the last one (tailwind-merge)', () => {
    expect(cn('px-2', 'px-4')).toBe('px-4');
    expect(cn('text-red-500', 'text-blue-500')).toBe('text-blue-500');
  });

  it('keeps non-conflicting classes from both sides', () => {
    expect(cn('px-2 py-1', 'text-sm')).toBe('px-2 py-1 text-sm');
  });

  it('a later conditional override wins even when applied through an object', () => {
    expect(cn('p-2', { 'p-4': true })).toBe('p-4');
  });
});
