import { type FormItem, FormItemType } from '@shared/presentation/ui';
import { t } from '@lingui/core/macro';

/** Built at call time: the caller rebuilds it on a locale switch. The camelCase file name keeps the Lingui ownership. */
export const createSecretsItems: () => readonly FormItem<'name' | 'description' | 'value'>[] =
  () => [
    {
      id: 'name',
      label: t`Secret name`,
      placeholder: t`e.g., DATABASE_URL`,
      type: FormItemType.Input,
      inputType: 'text',
      // The backend rule (scylla-domain secret/name.rs).
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
