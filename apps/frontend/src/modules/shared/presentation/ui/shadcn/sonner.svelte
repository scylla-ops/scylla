<script lang="ts">
  import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
  import InfoIcon from '@lucide/svelte/icons/info';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import OctagonXIcon from '@lucide/svelte/icons/octagon-x';
  import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
  import { Toaster as Sonner, type ToasterProps } from 'svelte-sonner';
  import { currentTheme } from '@shared/presentation/stores/theme.store.ts';

  let { ...rest }: ToasterProps = $props();
</script>

<Sonner
  theme={currentTheme()}
  class="toaster group"
  style="--normal-bg: var(--popover); --normal-text: var(--popover-foreground); --normal-border: var(--border); --border-radius: var(--radius);"
  toastOptions={{
    classes: {
      toast:
        'group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg',
      description: 'group-[.toast]:text-muted-foreground',
      actionButton: 'group-[.toast]:bg-primary group-[.toast]:text-primary-foreground',
      cancelButton: 'group-[.toast]:bg-muted group-[.toast]:text-muted-foreground',
    },
  }}
  {...rest}
>
  {#snippet successIcon()}
    <CircleCheckIcon class="size-4" />
  {/snippet}
  {#snippet infoIcon()}
    <InfoIcon class="size-4" />
  {/snippet}
  {#snippet warningIcon()}
    <TriangleAlertIcon class="size-4" />
  {/snippet}
  {#snippet errorIcon()}
    <OctagonXIcon class="size-4" />
  {/snippet}
  {#snippet loadingIcon()}
    <Loader2Icon class="size-4 animate-spin" />
  {/snippet}
</Sonner>
