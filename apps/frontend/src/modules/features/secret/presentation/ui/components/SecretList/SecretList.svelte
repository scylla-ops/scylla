<script lang="ts">
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import { i18n } from '@lingui/core';
  import { createMutation } from '@platform/query';
  import { Button } from '@shadcn';
  import { ConfirmOperationAlertDialog, CopyableText, DataTable } from '@shared/presentation/ui';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { formatDay } from '@shared/utils/date-utils.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { SecretEntity } from '../../../../domain/entities/secret.entity.ts';
  import { secretMutations } from '../../../secret.queries.ts';
  import { secretMessages } from '../../secret.messages.ts';
  import { secretColumns } from '../secret-columns.ts';

  interface Props {
    secrets: SecretEntity[];
    projectId: string;
  }

  let { secrets, projectId }: Props = $props();

  const selection = createSelection('secrets');
  const deleteSecret = createMutation(() => secretMutations.remove(projectId));

  /** The row awaiting confirmation. `null` closes the dialog. */
  let pendingDeletion = $state<string | null>(null);

  const confirmDelete = () => {
    const secretId = pendingDeletion;
    if (!secretId) return;

    deleteSecret.mutate(secretId, {
      onSuccess: () => toast.success(i18n._(ToastMessages.SECRET_DELETE)),
    });
    // Clicking the row's trash never selected it, so this drops it from the
    // selection if it was there and is a no-op otherwise.
    selection.select(secretId);
    pendingDeletion = null;
  };

  const columns = $derived(
    secretColumns({ name: nameCell, description: descriptionCell, createdAt: createdCell, actions: actionsCell }),
  );
</script>

{#snippet nameCell(secret: SecretEntity)}
  <div class="flex w-full min-w-0 flex-row items-center gap-4">
    <div class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
      <KeyRoundIcon class="size-4 text-primary" />
    </div>
    <div class="flex min-w-0 flex-col text-start">
      <p class="truncate font-semibold text-foreground">{secret.name}</p>
      <!-- The id, not the value: a secret's plaintext never reaches the client. -->
      <CopyableText class="text-xs text-muted-foreground/80" value={secret.id} />
    </div>
  </div>
{/snippet}

{#snippet descriptionCell(secret: SecretEntity)}
  <p class="truncate text-xs text-muted-foreground">{secret.description}</p>
{/snippet}

{#snippet createdCell(secret: SecretEntity)}
  <span class="text-sm whitespace-nowrap text-muted-foreground">{formatDay(secret.createdAt)}</span>
{/snippet}

{#snippet actionsCell(secret: SecretEntity)}
  <Button
    type="button"
    variant="ghost"
    size="icon"
    class="size-8"
    onclick={event => {
      // The row itself toggles selection on click; deleting must not also do it.
      event.stopPropagation();
      pendingDeletion = secret.id;
    }}
  >
    <Trash2Icon class="size-4 text-destructive" />
    <span class="sr-only">{t(secretMessages.deleteSecret)}</span>
  </Button>
{/snippet}

<DataTable
  {columns}
  data={secrets}
  isRowSelected={secret => selection.selectedIds.includes(secret.id)}
  onRowClick={row => selection.select(row.original.id)}
  getRowId={secret => secret.id}
  alignColumnsCenter
/>

<ConfirmOperationAlertDialog
  open={pendingDeletion !== null}
  onOpenChange={open => {
    if (!open) pendingDeletion = null;
  }}
  onContinue={confirmDelete}
/>
