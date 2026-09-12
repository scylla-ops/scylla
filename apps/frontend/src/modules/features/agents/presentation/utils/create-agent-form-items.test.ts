import { describe, it, expect } from 'vitest';
import { createAgentItems } from './create-agent-form-items';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';

describe('createAgentItems', () => {
  const [item] = createAgentItems();
  if (item.type !== FormItemType.Input) throw new Error('expected an Input item');
  const pattern = new RegExp(item.pattern!);

  it('accepts an alphanumeric name starting with a letter or digit, hyphens allowed after', () => {
    expect(pattern.test('my-build-runner')).toBe(true);
    expect(pattern.test('Runner1')).toBe(true);
  });

  it('rejects a name starting with a hyphen, or containing anything but [A-Za-z0-9-]', () => {
    expect(pattern.test('-runner')).toBe(false);
    expect(pattern.test('runner_1')).toBe(false);
    expect(pattern.test('runner 1')).toBe(false);
  });

  it('caps the name at 64 characters', () => {
    expect(pattern.test('a'.repeat(64))).toBe(true);
    expect(pattern.test('a'.repeat(65))).toBe(false);
  });
});
