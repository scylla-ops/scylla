import { describe, it, expect } from 'vitest';
import { createSecretsItems } from './createSecretItems';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';

describe('createSecretsItems', () => {
  const [item] = createSecretsItems();
  if (item.type !== FormItemType.Input) throw new Error('expected an Input item');
  const pattern = new RegExp(item.pattern!);

  it('mirrors the backend secret-name rule: alphanumeric plus . _ -', () => {
    expect(pattern.test('DATABASE_URL')).toBe(true);
    expect(pattern.test('my.secret-name_1')).toBe(true);
  });

  it('rejects anything outside that charset (e.g. a space or a slash)', () => {
    expect(pattern.test('DATABASE URL')).toBe(false);
    expect(pattern.test('path/to/secret')).toBe(false);
  });

  it('declares name, description, and value, in that order', () => {
    expect(createSecretsItems().map(i => i.id)).toEqual(['name', 'description', 'value']);
  });
});
