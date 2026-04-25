import type { ReactNode } from 'react';

export enum FormItemType {
  Input = 'input',
  Select = 'select',
}

export type FormInput = {
  type: FormItemType.Input;
  inputType: 'text' | 'password' | 'email' | 'tel';
};

export type SelectOption = {
  label: ReactNode;
  value: string;
};

export type FormSelect = {
  type: FormItemType.Select;
  options: SelectOption[];
};

export type FormItemBase = {
  label: ReactNode;
  placeholder?: string;
  id: string;
  className?: string;
  disabled?: boolean;
};

export type FormItem = FormItemBase & (FormInput | FormSelect);

export type FormChange = {
  id: string;
  value: string;
};
