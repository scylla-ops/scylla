<script lang="ts">
  import CheckIcon from '@lucide/svelte/icons/check';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  // Not the group barrel: it would import this file back.
  import IconButton from '../../controls/IconButton/IconButton.svelte';
  import { copyableTextMessages } from '../copyable-text.messages.ts';

  interface Props {
    value: string;
    truncate?: number;
    /** Defaults to `value`, truncated when `truncate` is set. */
    display?: string;
    showFullOnHover?: boolean;
    /** Default: "Copy". */
    copyLabel?: string;
    class?: string;
    /** E.g. to shrink the button inside a badge. */
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
    // Restart the timer, so a second click keeps the checkmark for its full delay.
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => (copied = false), 2000);
  };

  const text = $derived(display ?? (truncate ? `${value.slice(0, truncate)}...` : value));
</script>

<!-- `min-w-0` lets the text ellipsize in a flex row. -->
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
