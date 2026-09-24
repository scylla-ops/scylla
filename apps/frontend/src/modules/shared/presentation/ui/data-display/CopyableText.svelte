<script lang="ts">
  import CheckIcon from '@lucide/svelte/icons/check';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  // Direct path, not the group barrel: importing it from here would loop back
  // through data-display.
  import IconButton from '../controls/IconButton.svelte';
  import { copyableTextMessages } from './copyable-text.messages.ts';

  interface Props {
    /** The full text written to the clipboard. */
    value: string;
    /** Truncate the displayed text to N characters (followed by an ellipsis). */
    truncate?: number;
    /** Override what is rendered; defaults to `value` (truncated when `truncate` is set). */
    display?: string;
    /** Show the full `value` in a tooltip when hovering the text. */
    showFullOnHover?: boolean;
    /** Tooltip label on the copy button (default: "Copy"). */
    copyLabel?: string;
    class?: string;
    /** Extra classes for the copy button — shrink it for a compact context like a badge. */
    copyButtonClass?: string;
  }

  let {
    value,
    truncate,
    display,
    showFullOnHover = false,
    copyLabel,
    class: className,
    copyButtonClass,
  }: Props = $props();

  let copied = $state(false);
  let resetTimer: ReturnType<typeof setTimeout> | undefined;

  const handleCopy = (event: MouseEvent) => {
    event.stopPropagation();
    void navigator.clipboard.writeText(value);
    copied = true;
    // Restarted rather than stacked: two clicks in a row must not let the first
    // timer flip the icon back while the second copy is still fresh.
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => (copied = false), 2000);
  };

  const text = $derived(display ?? (truncate ? `${value.slice(0, truncate)}...` : value));
</script>

<!--
  Inline monospace text with a copy-to-clipboard button. The button toggles to a
  checkmark for 2s after copying. Used in table cells (job ids, webhook urls) so
  the copy affordance is identical everywhere instead of hand-rolled per cell.

  `min-w-0` on the root so the label can actually ellipsize in a flex row.
-->
<div class={cn('flex min-w-0 items-center gap-2', className)}>
  {#if showFullOnHover}
    <Tooltip>
      <TooltipTrigger class="truncate text-left font-mono">{text}</TooltipTrigger>
      <TooltipContent>
        <p>{value}</p>
      </TooltipContent>
    </Tooltip>
  {:else}
    <span class="truncate font-mono">{text}</span>
  {/if}
  <IconButton
    icon={copied ? CheckIcon : CopyIcon}
    tooltip={copied ? t(copyableTextMessages.copied) : (copyLabel ?? t(copyableTextMessages.copy))}
    onclick={handleCopy}
    class={cn('shrink-0', copyButtonClass)}
    iconClass={copied ? 'text-status-passed' : undefined}
  />
</div>
