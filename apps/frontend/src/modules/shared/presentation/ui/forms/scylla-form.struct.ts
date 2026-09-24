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

/** One string per item id. `TId` comes from the `items`, so a typo is a type error. */
export type FormValues<TId extends string = string> = Record<TId, string>;
