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
    /** Truncate to N chars with an ellipsis (full id when omitted). */
    truncate?: number;
    /** Chip variant: paperWarm background + faint border at rest. */
    chip?: boolean;
    class?: string;
  }

  let { id, truncate, chip = false, class: className }: Props = $props();

  const label = $derived(truncate && id.length > truncate ? `${id.slice(0, truncate)}…` : id);

  const copyId = async (event: MouseEvent) => {
    // Cards navigate on click — copying must not also open the agent.
    event.stopPropagation();
    try {
      await navigator.clipboard.writeText(id);
      toast.success(i18n._(ToastMessages.AGENT_ID_COPIED));
    } catch {
      // Clipboard can be denied (permissions, non-secure context) — say so
      // instead of failing silently.
      toast.error(i18n._(ToastMessages.AGENT_ID_COPY_ERROR));
    }
  };
</script>

<!--
  The agent id, click-to-copy: one click puts the full id in the clipboard and
  confirms with a toast. Shown truncated in tight spots (cards), full in the
  detail strip — the copy always carries the complete id.
-->
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
