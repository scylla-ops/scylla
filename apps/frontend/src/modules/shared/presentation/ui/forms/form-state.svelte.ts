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
 * Values, changes, reset and validity of a declarative form. `items` typed with
 * literal ids types `values` (`values.username`). `items` is a getter; the values
 * are seeded once, so what the user typed survives a change of the items.
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
      // Only inputs have a pattern; a select picks from a closed list.
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
