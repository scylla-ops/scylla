<script lang="ts">
  import { createMutation } from '@platform/query';
  import { FormDialog, type FormValues } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { activeLocale } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretMutations } from '../../secret.queries.ts';
  import { createSecretsItems } from '../../utils/createSecretItems.ts';
  import { secretMessages } from '../secret.messages.ts';

  interface Props {
    projectId: string;
    isOpen: boolean;
    setOpen: (open: boolean) => void;
  }

  let { projectId, isOpen, setOpen }: Props = $props();

  const createSecret = createMutation(() => secretMutations.create(projectId));

  // Reading the locale rebuilds the labels on a switch.
  const items = $derived((activeLocale(), createSecretsItems()));

  const handleSubmit = ({
    name,
    description,
    value,
  }: FormValues<'name' | 'description' | 'value'>) => {
    // Rejects whitespace-only input.
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
