/**
 * The declarative shape a Svelte form is built from.
 *
 * Colocated with its consumers rather than sitting in `presentation/structs/`
 * beside the React original: the two differ — every `ReactNode` here is a
 * `string`, because a Svelte component resolves `<Trans>` to text through
 * `t()` before it ever reaches a prop — and two files one letter apart in the
 * same folder is how a feature ends up importing the wrong `FormItemType`.
 * In Phase 6 the React struct goes and this one moves up.
 */
export enum FormItemType {
  Input = 'input',
  Select = 'select',
}

export type FormInput = {
  type: FormItemType.Input;
  inputType: 'text' | 'password' | 'email' | 'tel';
  pattern?: string;
};

export type SelectOption = {
  label: string;
  value: string;
};

export type FormSelect = {
  type: FormItemType.Select;
  options: SelectOption[];
};

export type FormItemBase<TId extends string = string> = {
  label: string;
  placeholder?: string;
  id: TId;
  class?: string;
  disabled?: boolean;
  optional?: boolean;
  defaultValue?: string;
};

export type FormItem<TId extends string = string> = FormItemBase<TId> & (FormInput | FormSelect);

/**
 * What a form holds and submits: one string per declared item id.
 *
 * `TId` is inferred from the `items` array, so a form declared with literal ids
 * submits `{ username: string; password: string }` and a typo is a type error.
 * Items typed as plain `FormItem[]` fall back to `Record<string, string>`.
 */
export type FormValues<TId extends string = string> = Record<TId, string>;
