<script lang="ts">
  import { i18n } from '@lingui/core';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import { cn } from '@shared/presentation/utils';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { agentsMessages } from '../../agents.messages.ts';

  interface Props {
    id: string;
    truncate?: number;
    chip?: boolean;
    class?: string;
  }

  let { id, truncate, chip = false, class: className }: Props = $props();

  const label = $derived(truncate && id.length > truncate ? `${id.slice(0, truncate)}…` : id);

  const copyId = async (event: MouseEvent) => {
    // The card opens on click: copying must not.
    event.stopPropagation();
    try {
      await navigator.clipboard.writeText(id);
      toast.success(i18n._(ToastMessages.AGENT_ID_COPIED));
    } catch {
      // The clipboard can be denied: say so.
      toast.error(i18n._(ToastMessages.AGENT_ID_COPY_ERROR));
    }
  };
</script>

<!-- Click to copy the full id, even when it shows truncated. -->
<button
  type="button"
  onclick={event => void copyId(event)}
  title={t(agentsMessages.copyAgentId)}
  aria-label={t(agentsMessages.copyAgentId)}
  class={cn(
    'group inline-flex cursor-pointer items-center gap-1 font-mono text-xs text-foreground transition-colors duration-100',
    'hover:text-success hover:underline hover:decoration-success hover:underline-offset-[3px]',
    chip &&
      'rounded border border-border bg-muted/60 px-1.5 py-0.5 hover:border-success/60 hover:bg-success/10',
    className,
  )}
>
  <span class="truncate">{label}</span>
  <CopyIcon
    class="h-2.5 w-2.5 shrink-0 opacity-55 transition-colors group-hover:text-success group-hover:opacity-100"
  />
</button>
