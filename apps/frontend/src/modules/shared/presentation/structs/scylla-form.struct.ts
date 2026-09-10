import type { ReactNode } from 'react';

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
  label: ReactNode;
  value: string;
};

export type FormSelect = {
  type: FormItemType.Select;
  options: SelectOption[];
};

export type FormItemBase<TId extends string = string> = {
  label: ReactNode;
  placeholder?: string;
  id: TId;
  className?: string;
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
