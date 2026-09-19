<script lang="ts">
  import { createMutation } from '@platform/query';
  import { FormDialog, type FormValues } from '@shared/presentation/ui-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { activeLocale } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretMutations } from '../secret.queries.ts';
  import { createSecretsItems } from '../utils/createSecretItems.ts';
  import { secretMessages } from './secret.messages.ts';

  interface Props {
    projectId: string;
    isOpen: boolean;
    setOpen: (open: boolean) => void;
  }

  let { projectId, isOpen, setOpen }: Props = $props();

  const createSecret = createMutation(() => secretMutations.create(projectId));

  // `createSecretsItems` resolves its labels with the Lingui macro at call
  // time, so reading the active locale here is what rebuilds them on a switch.
  const items = $derived((activeLocale(), createSecretsItems()));

  const handleSubmit = ({
    name,
    description,
    value,
  }: FormValues<'name' | 'description' | 'value'>) => {
    // The form already blocks an empty required field; this is the guard
    // against whitespace-only input reaching the backend.
    if (!name.trim() || !value.trim()) return;

    createSecret.mutate({ name, description: description.trim(), value });
    setOpen(false);
  };
</script>

<FormDialog
  open={isOpen}
  onOpenChange={setOpen}
  title={t(secretMessages.createSecret)}
  {items}
  onSubmit={handleSubmit}
/>
