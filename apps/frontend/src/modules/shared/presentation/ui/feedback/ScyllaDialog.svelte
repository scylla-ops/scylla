<script lang="ts">
  import type { Snippet } from 'svelte';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '@shadcn';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title?: string;
    description?: string;
    /** Replaces the title and the description, for a header with more than text. */
    header?: Snippet;
    /** When false: no close button, and Escape or a click outside does nothing. */
    dismissible?: boolean;
    class?: string;
    /** Built again at each opening, so a form in it starts empty. */
    children: Snippet;
  }

  let {
    open,
    onOpenChange,
    title,
    description,
    header,
    dismissible = true,
    class: className,
    children,
  }: Props = $props();

  let openings = 0;
  const opening = $derived(open ? ++openings : openings);

  const keepOpen = (event: Event) => {
    if (!dismissible) event.preventDefault();
  };
</script>

<Dialog {open} {onOpenChange}>
  <DialogContent
    class={cn(!dismissible && '[&>button]:hidden', className)}
    onEscapeKeydown={keepOpen}
    onInteractOutside={keepOpen}
  >
    {#if header}
      {@render header()}
    {:else if title}
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
        {#if description}
          <DialogDescription>{description}</DialogDescription>
        {/if}
      </DialogHeader>
    {/if}
    {#key opening}
      {@render children()}
    {/key}
  </DialogContent>
</Dialog>
