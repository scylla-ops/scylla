import { describe, it, expect } from 'vitest';
import { createAppItems } from './create-app-form-items';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';

describe('createAppItems', () => {
  const [item] = createAppItems();
  if (item.type !== FormItemType.Input) throw new Error('expected an Input item');
  const pattern = new RegExp(item.pattern!);

  it('is lowercase-only, unlike the agent name pattern', () => {
    expect(pattern.test('my-build-runner')).toBe(true);
    expect(pattern.test('My-Runner')).toBe(false);
  });

  it('rejects a name starting with a hyphen', () => {
    expect(pattern.test('-runner')).toBe(false);
  });

  it('caps the name at 64 characters', () => {
    expect(pattern.test('a'.repeat(64))).toBe(true);
    expect(pattern.test('a'.repeat(65))).toBe(false);
  });
});
