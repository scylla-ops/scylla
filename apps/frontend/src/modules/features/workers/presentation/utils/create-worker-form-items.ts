import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createWorkerItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`Worker name`,
    placeholder: t`e.g., ci-runner`,
    type: FormItemType.Input,
    inputType: 'text',
  },
];
