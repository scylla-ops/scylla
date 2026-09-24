<script lang="ts">
  import type { Snippet } from 'svelte';
  import CheckIcon from '@lucide/svelte/icons/check';
  import EyeIcon from '@lucide/svelte/icons/eye';
  import {
    Button,
    CodeSnippet,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretRevealMessages } from './secret-reveal.messages.ts';

  interface Props {
    title: string;
    description: string;
    secret: string;
    secretLabel: string;
    copyToast?: string;
    secretStepTitle?: string;
    secondStep?: { title: string; content: Snippet };
    revealedNote?: string;
    footerNote?: string;
    onClose: () => void;
  }

  let {
    title,
    description,
    secret,
    secretLabel,
    copyToast,
    secretStepTitle,
    secondStep,
    revealedNote,
    footerNote,
    onClose,
  }: Props = $props();

  /** Resets at each opening: the dialog rebuilds its content. */
  let revealed = $state(false);

  let confirmButton = $state<HTMLElement | null>(null);

  const reveal = () => {
    revealed = true;
    // Focus the only action left. The button already exists, it was only disabled.
    confirmButton?.focus();
  };
</script>

<DialogHeader class="space-y-1 p-5 pb-3 text-left">
  <DialogTitle class="text-[15px]">{title}</DialogTitle>
  <DialogDescription class="text-[13px]">{description}</DialogDescription>
</DialogHeader>

<div class="space-y-1 px-5 pb-4">
  <div class="flex gap-3">
    <div class="flex flex-col items-center">
      {@render stepBullet(1, true)}
      {#if secondStep}
        <span class="mt-1.5 w-px flex-1 bg-border"></span>
      {/if}
    </div>
    <div class="min-w-0 flex-1 pb-4">
      <p class="mb-2 mt-0.5 text-[13px] font-medium">
        {secretStepTitle ?? t(secretRevealMessages.copySecret)}
        <span class="font-normal text-muted-foreground">
          · {t(secretRevealMessages.shownOnce)}
        </span>
      </p>
      <CodeSnippet
        multiline
        value={secret}
        blurred={!revealed}
        copyToast={copyToast ?? t(secretRevealMessages.secretCopied)}
        label={secretLabelSnippet}
        overlay={revealOverlay}
      />
    </div>
  </div>

  {#if secondStep}
    <div class="flex gap-3">
      <div class="flex flex-col items-center">
        {@render stepBullet(2, revealed)}
      </div>
      <div class={cn('min-w-0 flex-1 transition-opacity', revealed ? 'opacity-100' : 'opacity-40')}>
        <p class="mb-2 mt-0.5 text-[13px] font-medium">{secondStep.title}</p>
        {#if revealed}{@render secondStep.content()}{/if}
      </div>
    </div>
  {:else if revealed && revealedNote}
    <p class="pl-9 text-xs text-muted-foreground">{revealedNote}</p>
  {/if}
</div>

<DialogFooter class="items-center justify-between gap-2 border-t p-4 sm:justify-between">
  <span class="text-xs text-muted-foreground">
    {footerNote ?? t(secretRevealMessages.noSecondChance)}
  </span>
  <Button
    bind:ref={confirmButton}
    disabled={!revealed}
    title={revealed ? undefined : t(secretRevealMessages.revealFirst)}
    onclick={onClose}
  >
    <CheckIcon class="mr-1.5 h-4 w-4" />
    {t(secretRevealMessages.done)}
  </Button>
</DialogFooter>

{#snippet secretLabelSnippet()}
  <span class="truncate font-mono">{secretLabel}</span>
{/snippet}

{#snippet revealOverlay()}
  <Button size="sm" variant="outline" autofocus onclick={reveal} class="gap-2">
    <EyeIcon class="h-4 w-4" />
    {t(secretRevealMessages.reveal)}
  </Button>
{/snippet}

{#snippet stepBullet(n: number, active: boolean)}
  <span
    class={cn(
      'flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-full text-[11px] font-medium transition-colors',
      active
        ? 'bg-primary text-primary-foreground'
        : 'border-[1.5px] border-border text-muted-foreground',
    )}
  >
    {n}
  </span>
{/snippet}
