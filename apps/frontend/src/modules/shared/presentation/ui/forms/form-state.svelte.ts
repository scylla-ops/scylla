import { FormItemType, type FormItem, type FormValues } from './scylla-form.struct.ts';

export interface FormState<TId extends string> {
  readonly values: FormValues<TId>;
  readonly isValid: boolean;
  handleChange: (id: TId, value: string) => void;
  reset: () => void;
}

const initialValues = <TId extends string>(items: readonly FormItem<TId>[]): FormValues<TId> =>
  Object.fromEntries(items.map(item => [item.id, item.defaultValue ?? ''])) as FormValues<TId>;

/**
 * Values, changes, reset and validity for a declarative form — the Svelte
 * counterpart of `useFormState`.
 *
 * The generic matters more than it looks: `items` typed
 * `readonly FormItem<'a' | 'b'>[]` makes `values` a `{ a: string; b: string }`,
 * so a consumer reads `values.username` instead of searching the array by id,
 * and a typo is a compile error. Losing `TId` to a bare `string` is the easiest
 * thing to get wrong in this port and nothing at runtime would notice.
 *
 * `isValid` is `$derived`, not a field kept in step by hand: it is a pure
 * function of the values and the item declarations, so there is nothing to
 * synchronise and no state that can disagree with what is on screen.
 *
 * `items` is a getter because it can be a prop — `FormDialog` passes one
 * straight through. Taking the array once would pin validation to whatever the
 * first render saw, which is the React version's behaviour only by accident
 * (it re-read `items` on every render). The *values* are still seeded once, as
 * they were: re-seeding them when the declarations change would wipe what the
 * user has typed.
 */
export const createFormState = <TId extends string>(
  items: () => readonly FormItem<TId>[],
): FormState<TId> => {
  let values = $state<FormValues<TId>>(initialValues(items()));

  const isValid = $derived(
    items().every(item => {
      if (item.optional) return true;
      const value = values[item.id] ?? '';
      if (value.trim().length === 0) return false;
      // Gated on `Input` specifically, not on "has a pattern field": a select
      // has no pattern to check, and its value comes from a closed list anyway.
      if (item.type !== FormItemType.Input || !item.pattern) return true;
      return new RegExp(item.pattern).test(value);
    }),
  );

  return {
    get values() {
      return values;
    },
    get isValid() {
      return isValid;
    },
    handleChange: (id: TId, value: string) => {
      values = { ...values, [id]: value };
    },
    reset: () => {
      values = initialValues(items());
    },
  };
};
