<script lang="ts">
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { Badge } from '@shadcn';
  import { IconButton } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    isCurrentUser: boolean;
    /** False when the caller may not remove the member, or nothing is removable. */
    canRemove: boolean;
    disabled?: boolean;
    tooltip: string;
    onRemove: () => void;
  }

  let { isCurrentUser, canRemove, disabled = false, tooltip, onRemove }: Props = $props();
</script>

<!-- No self-removal: it would lock the caller out mid-session. -->
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
