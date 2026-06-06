import { type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { t } from '@lingui/core/macro';

/**
 * Fields for the "Create secret" dialog. The value is a password input — it is
 * write-only, masked on entry, and never read back from the server.
 */
export const createSecretItems = (): FormItem[] => [
  {
    id: 'name',
    label: t`Secret name`,
    placeholder: t`DATABASE_URL`,
    type: FormItemType.Input,
    inputType: 'text',
    pattern: '^[A-Za-z_][A-Za-z0-9_]*$',
  },
  {
    id: 'value',
    label: t`Value`,
    placeholder: t`The secret value`,
    type: FormItemType.Input,
    inputType: 'password',
  },
  {
    id: 'description',
    label: t`Description`,
    placeholder: t`What this secret is for`,
    type: FormItemType.Input,
    inputType: 'text',
    optional: true,
  },
];
