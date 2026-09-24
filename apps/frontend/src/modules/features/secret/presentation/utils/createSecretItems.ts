import { type FormItem, FormItemType } from '@shared/presentation/ui';
import { t } from '@lingui/core/macro';

/**
 * The create-secret form, as data.
 *
 * Still `t` from the Lingui macro rather than the Svelte `t()`: this returns a
 * plain array built at call time, and the Svelte `FormItem` takes `string`
 * labels where the React one took `ReactNode`. The caller rebuilds the items
 * when the locale changes.
 *
 * The filename stays camelCase — renaming it would move Lingui's message
 * ownership and need `scripts/restore-translations.mjs`. New files are
 * kebab-case.
 */
export const createSecretsItems: () => readonly FormItem<'name' | 'description' | 'value'>[] =
  () => [
    {
      id: 'name',
      label: t`Secret name`,
      placeholder: t`e.g., DATABASE_URL`,
      type: FormItemType.Input,
      inputType: 'text',
      // Mirrors the backend rule (scylla-domain secret/name.rs): alphanumeric, '-', '_', '.'
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
