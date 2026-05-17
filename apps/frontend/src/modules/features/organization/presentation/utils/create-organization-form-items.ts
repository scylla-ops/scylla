import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

export const createOrganizationItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`Organization name`,
    placeholder: t`e.g., My Organization`,
    type: FormItemType.Input,
    inputType: 'text',
  },
  {
    id: 'description',
    label: t`Description`,
    placeholder: t`e.g., Our company's main organization`,
    type: FormItemType.Input,
    inputType: 'text',
  },
];
