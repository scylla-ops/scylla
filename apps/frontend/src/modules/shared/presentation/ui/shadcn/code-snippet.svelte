<script lang="ts">
  import type { Snippet } from 'svelte';
  import { i18n } from '@lingui/core';
  import CheckIcon from '@lucide/svelte/icons/check';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import { cn } from '@shared/presentation/utils';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { codeSnippetMessages } from './code-snippet.messages.ts';

  type CodeSnippetVariant = 'default' | 'dark' | 'warning';

  interface Props {
    /** Text shown and copied. */
    value: string;
    /** Header label on the left of the top bar. */
    label?: Snippet;
    variant?: CodeSnippetVariant;
    /** Wrap long lines instead of letting them overflow. */
    multiline?: boolean;
    /** Blur the body + disable copy (one-time-secret pattern). */
    blurred?: boolean;
    /** Centered overlay over the body (e.g. a "reveal" button) while blurred. */
    overlay?: Snippet;
    /** Toast message on successful copy. */
    copyToast?: string;
    class?: string;
  }

  let {
    value,
    label,
    variant = 'default',
    multiline = false,
    blurred = false,
    overlay,
    copyToast,
    class: className,
  }: Props = $props();

  const VARIANT: Record<
    CodeSnippetVariant,
    { box: string; header: string; body: string; copy: string }
  > = {
    default: {
      box: 'border bg-muted/50',
      header: 'border-border/60 text-muted-foreground',
      body: 'text-foreground',
      copy: 'text-muted-foreground hover:bg-foreground/10',
    },
    dark: {
      box: 'border-transparent bg-foreground',
      header: 'border-background/15 text-background/60',
      body: 'text-background',
      copy: 'text-background/70 hover:bg-background/15',
    },
    warning: {
      box: 'border-warning/40 bg-warning/10',
      header: 'border-warning/30 text-warning',
      body: 'text-foreground',
      copy: 'text-warning hover:bg-warning/20',
    },
  };

  let copied = $state(false);
  let resetTimer: ReturnType<typeof setTimeout> | undefined;

  const styles = $derived(VARIANT[variant]);

  const copy = async () => {
    if (blurred) return;

    await navigator.clipboard.writeText(value);
    copied = true;
    toast.success(copyToast ?? i18n._(ToastMessages.COPIED));
    // Restarted rather than stacked, as in `CopyableText`: two copies in a row
    // must not let the first timer flip the icon back while the second is fresh.
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => (copied = false), 1500);
  };
</script>

<!--
  A code/credential box: a thin header bar carrying an optional label and a copy
  button, with a monospace body beneath. One component so the copy affordance is
  consistent everywhere instead of hand-rolled per call site.
-->
<div class={cn('overflow-hidden rounded-md border', styles.box, className)}>
  <div class={cn('flex items-center justify-between gap-2 border-b px-3 py-1.5', styles.header)}>
    <span class="font-mono text-[10px] uppercase tracking-wide">{@render label?.()}</span>
    <button
      type="button"
      aria-label={t(codeSnippetMessages.copy)}
      onclick={() => void copy()}
      disabled={blurred}
      class={cn(
        'rounded p-1 transition-colors disabled:cursor-not-allowed disabled:opacity-40',
        styles.copy,
      )}
    >
      {#if copied}
        <CheckIcon class="h-3.5 w-3.5" />
      {:else}
        <CopyIcon class="h-3.5 w-3.5" />
      {/if}
    </button>
  </div>
  <div class="relative">
    <pre
      class={cn(
        'px-3 py-2.5 font-mono text-xs',
        multiline ? 'whitespace-pre-wrap break-all' : 'overflow-x-auto',
        styles.body,
        blurred && 'secret-blur',
      )}>{value}</pre>
    {#if blurred && overlay}
      <div class="absolute inset-0 flex items-center justify-center">{@render overlay()}</div>
    {/if}
  </div>
</div>
