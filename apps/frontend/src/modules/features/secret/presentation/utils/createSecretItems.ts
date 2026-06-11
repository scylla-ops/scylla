import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createSecretsItems: () => FormItem[] = () => {
  return [
    {
      id: 'name',
      label: t`Secret name`,
      placeholder: t`e.g., My secret`,
      type: FormItemType.Input,
      inputType: 'text',
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
