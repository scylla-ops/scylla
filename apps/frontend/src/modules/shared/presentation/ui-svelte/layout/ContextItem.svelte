<script lang="ts">
  import type { LucideIcon } from '../icon.ts';

  interface Props {
    name: string;
    description?: string;
    icon: LucideIcon;
  }

  let { name, description, icon: Icon }: Props = $props();
</script>

<div
  class="group flex w-full cursor-pointer items-center gap-3 rounded-lg px-2 py-2 text-sm transition-colors hover:bg-accent"
>
  <div
    class="flex size-8 items-center justify-center rounded-md border border-border bg-background transition-colors group-hover:border-primary/50"
  >
    <Icon class="size-4 text-muted-foreground transition-colors group-hover:text-primary" />
  </div>

  <div class="min-w-0 flex-1">
    <span
      class="block truncate font-medium text-foreground transition-colors group-hover:text-primary"
    >
      {name}
    </span>
    {#if description}
      <span class="block truncate text-xs text-muted-foreground">{description}</span>
    {/if}
  </div>
</div>
