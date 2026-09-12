import { describe, it, expect } from 'vitest';
import { createAppSecretItems } from './create-app-secret-form-items';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';

describe('createAppSecretItems', () => {
  const [item] = createAppSecretItems();
  if (item.type !== FormItemType.Input) throw new Error('expected an Input item');
  const pattern = new RegExp(item.pattern!);

  it('accepts a lowercase kebab-case label', () => {
    expect(pattern.test('ci-runner')).toBe(true);
  });

  it('rejects uppercase and a leading hyphen', () => {
    expect(pattern.test('CI-Runner')).toBe(false);
    expect(pattern.test('-ci-runner')).toBe(false);
  });
});
