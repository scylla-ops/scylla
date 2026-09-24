// @vitest-environment node
import { describe, it, expect, expectTypeOf } from 'vitest';
import { createFormState } from './form-state.svelte.ts';
import { FormItemType, type FormItem } from './scylla-form.struct.ts';

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

describe('createFormState', () => {
  it("seeds values from each item's defaultValue, or empty string when none", () => {
    const withDefaults: readonly FormItem<'a' | 'b'>[] = [
      { id: 'a', type: FormItemType.Input, inputType: 'text', label: 'A', defaultValue: 'hi' },
      { id: 'b', type: FormItemType.Input, inputType: 'text', label: 'B' },
    ];

    expect(createFormState(() => withDefaults).values).toEqual({ a: 'hi', b: '' });
  });

  it('updates only the targeted field on handleChange', () => {
    const form = createFormState(() => items);

    form.handleChange('username', 'ravenne');

    expect(form.values.username).toBe('ravenne');
    expect(form.values.bio).toBe('');
  });

  it('restores every field to its default on reset', () => {
    const form = createFormState(() => items);

    form.handleChange('username', 'ravenne');
    form.reset();

    expect(form.values).toEqual({ username: '', bio: '', email: '' });
  });

  it('is invalid while a required field is empty', () => {
    expect(createFormState(() => items).isValid).toBe(false);
  });

  it('does not let an empty optional field block validity', () => {
    const onlyOptional: readonly FormItem<'bio'>[] = [
      { id: 'bio', type: FormItemType.Input, inputType: 'text', label: 'Bio', optional: true },
    ];

    expect(createFormState(() => onlyOptional).isValid).toBe(true);
  });

  it('still counts a required field that is only whitespace as empty', () => {
    const form = createFormState(() => items);

    form.handleChange('username', '   ');
    form.handleChange('email', 'a@b.co');

    expect(form.isValid).toBe(false);
  });

  it('enforces the item pattern once a value is present', () => {
    const form = createFormState(() => items);
    form.handleChange('username', 'ravenne');

    form.handleChange('email', 'not-an-email');
    expect(form.isValid).toBe(false);

    form.handleChange('email', 'ravenne@scylla.dev');
    expect(form.isValid).toBe(true);
  });

  it('never pattern-checks a select, whose value comes from a closed list', () => {
    const selectItem: readonly FormItem<'plan'>[] = [
      {
        id: 'plan',
        type: FormItemType.Select,
        label: 'Plan',
        options: [{ label: 'Free', value: 'free' }],
        defaultValue: 'free',
      },
    ];

    expect(createFormState(() => selectItem).isValid).toBe(true);
  });

  it('re-reads the item declarations, so a form whose items arrive later validates against them', () => {
    let declarations: readonly FormItem<'name'>[] = [
      { id: 'name', type: FormItemType.Input, inputType: 'text', label: 'Name', optional: true },
    ];
    const form = createFormState(() => declarations);
    expect(form.isValid).toBe(true);

    declarations = [{ id: 'name', type: FormItemType.Input, inputType: 'text', label: 'Name' }];

    expect(form.isValid).toBe(false);
  });
});

/**
 * The generic is the whole point of the declarative form, and it is the thing a
 * port loses silently: widen `TId` to `string` and every test above still
 * passes, while consumers go back to indexing values by a stringly-typed id.
 */
describe('the id generic', () => {
  it('turns literal item ids into a typed values record', () => {
    const form = createFormState(() => items);

    expectTypeOf(form.values).toEqualTypeOf<Record<Ids, string>>();
    expectTypeOf(form.values.username).toEqualTypeOf<string>();
    // @ts-expect-error — 'nickname' was never declared as an item id.
    expectTypeOf(form.values.nickname);
  });

  it('rejects a change to an id the form does not declare', () => {
    const form = createFormState(() => items);

    expectTypeOf(form.handleChange).parameter(0).toEqualTypeOf<Ids>();
    // @ts-expect-error — same reason: the id is checked, not just the value.
    form.handleChange('nickname', 'x');
  });
});
