import { FormItemType, type FormItem } from '../scylla-form.struct.ts';

export type Ids = 'username' | 'bio';

export const items: readonly FormItem<Ids>[] = [
  { id: 'username', type: FormItemType.Input, inputType: 'text', label: 'Username' },
  { id: 'bio', type: FormItemType.Input, inputType: 'text', label: 'Bio', optional: true },
];
