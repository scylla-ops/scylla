import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createAppItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`App name`,
    placeholder: t`e.g., ci-runner`,
    type: FormItemType.Input,
    inputType: 'text',
  },
];
