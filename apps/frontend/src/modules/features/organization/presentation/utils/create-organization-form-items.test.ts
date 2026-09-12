import { describe, it, expect } from 'vitest';
import { createOrganizationItems } from './create-organization-form-items';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';

describe('createOrganizationItems', () => {
  it('declares a name and a description field, neither pattern-constrained', () => {
    const items = createOrganizationItems();
    expect(items.map(i => i.id)).toEqual(['name', 'description']);
    expect(
      items.every(i => i.type === FormItemType.Input && i.pattern === undefined),
    ).toBe(true);
  });
});
