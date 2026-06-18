import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createAgentItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`Name`,
    placeholder: t`my-build-runner`,
    type: FormItemType.Input,
    inputType: 'text',
    pattern: '^[a-zA-Z0-9][a-zA-Z0-9-]{0,63}$',
  },
];
