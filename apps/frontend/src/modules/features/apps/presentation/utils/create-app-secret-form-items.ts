import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createAppSecretItems = (): FormItem[] => [
  {
    id: 'label',
    label: t`Label`,
    placeholder: t`ci-runner`,
    type: FormItemType.Input,
    inputType: 'text',
    pattern: '^[a-z0-9][a-z0-9-]{0,63}$',
  },
];
