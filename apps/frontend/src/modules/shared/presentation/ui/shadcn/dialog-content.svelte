<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Dialog as DialogPrimitive } from 'bits-ui';
  import XIcon from '@lucide/svelte/icons/x';
  import { cn } from '@shared/presentation/utils';
  import DialogOverlay from './dialog-overlay.svelte';

  type Props = DialogPrimitive.ContentProps & { children?: Snippet };

  let { class: className, children, onOpenAutoFocus, ref = $bindable(null), ...rest }: Props =
    $props();

  /**
   * Puts the caret in the first field, the way Radix did.
   *
   * Radix's focus scope focused the first *tabbable* element in the content;
   * bits-ui focuses the content element itself, so a form dialog opened with
   * nothing focused and the user had to tab into it. `ScyllaForm` marks its
   * first input `autofocus`, and this honours it — when nothing asks,
   * bits-ui's own default stands.
   */
  const focusFirstField = (event: Event) => {
    onOpenAutoFocus?.(event);
    if (event.defaultPrevented) return;

    const first = ref?.querySelector<HTMLElement>('[autofocus]');
    if (!first) return;

    event.preventDefault();
    first.focus();
  };
</script>

<DialogPrimitive.Portal>
  <DialogOverlay />
  <DialogPrimitive.Content
    bind:ref
    onOpenAutoFocus={focusFirstField}
    data-slot="dialog-content"
    class={cn(
      'fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 max-h-[calc(100dvh-2rem)] overflow-y-auto border border-border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 sm:rounded-lg',
      className,
    )}
    {...rest}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-foreground focus:ring-offset-2 disabled:pointer-events-none"
    >
      <XIcon class="h-4 w-4" />
      <!-- The only accessible name this control has: the icon carries none. -->
      <span class="sr-only">Close</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
