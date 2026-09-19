<script lang="ts">
  import EyeIcon from '@lucide/svelte/icons/eye';
  import { Avatar, AvatarFallback, AvatarImage } from '@shadcn-svelte';
  import { DataTable, IconButton, TruncatedText } from '@shared/presentation/ui-svelte';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { UserEntity } from '../../../../domain/entities/user.entity.ts';
  import { userMessages } from '../../user.messages.ts';
  import { userColumns } from './user-columns.ts';

  interface Props {
    data?: UserEntity[];
    onView: (userId: string) => void;
  }

  let { data, onView }: Props = $props();

  const selection = createSelection('users');

  const columns = $derived(
    userColumns({ username: usernameCell, creationDate: createdCell, actions: actionsCell }),
  );
</script>

{#snippet usernameCell(user: UserEntity)}
  <div class="flex w-full min-w-0 flex-row items-center gap-2">
    <Avatar class="h-8 w-8 shrink-0 rounded-lg">
      <AvatarImage />
      <AvatarFallback class="rounded-lg">{user.username.at(0)?.toUpperCase()}</AvatarFallback>
    </Avatar>
    <TruncatedText tooltip={user.username} class="text-xs">{user.username}</TruncatedText>
  </div>
{/snippet}

{#snippet createdCell(user: UserEntity)}
  <span class="truncate">{formatDate(user.createdAt)}</span>
{/snippet}

{#snippet actionsCell(user: UserEntity)}
  <IconButton
    icon={EyeIcon}
    tooltip={t(userMessages.view)}
    onclick={event => {
      // The row click selects; viewing must not also toggle the selection.
      event.stopPropagation();
      onView(user.userId);
    }}
  />
{/snippet}

<DataTable
  {columns}
  data={data ?? []}
  onRowClick={row => selection.select(row.original.userId)}
  getRowId={(user, index) => user.userId || index.toString()}
  isRowSelected={user => selection.selectedIds.includes(user.userId)}
  alignRowsCenter
/>
