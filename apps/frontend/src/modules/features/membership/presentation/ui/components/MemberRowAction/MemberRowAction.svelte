<script lang="ts">
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { Badge } from '@shadcn';
  import { IconButton } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    /** The signed-in user's own row: marked, never removable. */
    isCurrentUser: boolean;
    /** False when the caller may not remove this member, or there is nothing to remove. */
    canRemove: boolean;
    disabled?: boolean;
    tooltip: string;
    onRemove: () => void;
  }

  let { isCurrentUser, canRemove, disabled = false, tooltip, onRemove }: Props = $props();
</script>

<!--
  The trailing control on a member row: remove them, or say that the row is you.

  Self-removal is excluded outright rather than offered and refused. It would
  revoke the caller's own access mid-session, and the backend may reject it
  anyway as the scope's last owner — a button whose only outcomes are "lock
  yourself out" and "error" is not a button.
-->
{#if isCurrentUser}
  <Badge variant="outline" class="text-[10px]">{t(membershipMessages.you)}</Badge>
{:else if canRemove}
  <IconButton
    icon={TrashIcon}
    {tooltip}
    {disabled}
    onclick={onRemove}
    class="size-8 hover:bg-destructive-subtle hover:text-destructive"
    iconClass="h-3.5 w-3.5"
  />
{/if}
