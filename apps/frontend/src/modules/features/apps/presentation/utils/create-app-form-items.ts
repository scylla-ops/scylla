import { type FormItem, FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';
import { t } from '@lingui/core/macro';

export const createAppItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`Name`,
    placeholder: t`my-build-runner`,
    type: FormItemType.Input,
    inputType: 'text',
    pattern: '^[a-z0-9][a-z0-9-]{0,63}$',
  },
];
