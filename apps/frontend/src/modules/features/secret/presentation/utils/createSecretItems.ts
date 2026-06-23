import { type FormItem, FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';
import { t } from '@lingui/core/macro';

export const createSecretsItems: () => FormItem[] = () => {
  return [
    {
      id: 'name',
      label: t`Secret name`,
      placeholder: t`e.g., DATABASE_URL`,
      type: FormItemType.Input,
      inputType: 'text',
      // Mirrors the backend rule (scylla-core secret/name.rs): alphanumeric, '-', '_', '.'
      pattern: '^[A-Za-z0-9._-]+$',
    },
    {
      id: 'description',
      label: t`Description`,
      placeholder: t`e.g., Our company's main secret`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'value',
      label: t`Value`,
      placeholder: t`Top secret value`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];
};
