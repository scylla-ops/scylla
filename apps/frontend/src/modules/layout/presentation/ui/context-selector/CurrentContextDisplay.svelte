<script lang="ts">
  import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
  import { getSidebar } from '@shadcn';
  import type { LucideIcon } from '@shared/presentation/ui/icon.ts';

  interface Props {
    name: string;
    description: string;
    icon: LucideIcon;
  }

  let { name, description, icon: Icon }: Props = $props();

  const sidebar = getSidebar();
</script>

{#if sidebar.state === 'collapsed'}
  <div
    class="flex size-8 cursor-pointer items-center justify-center rounded-lg bg-primary text-primary-foreground"
  >
    <Icon class="size-4" strokeWidth={2.5} />
  </div>
{:else}
  <div
    class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-sm"
  >
    <Icon class="size-4" strokeWidth={2.5} />
  </div>

  <div class="min-w-0 flex-1 text-left">
    <p class="truncate text-sm font-semibold leading-tight text-foreground">{name}</p>
    <p class="truncate text-xs leading-tight text-muted-foreground">{description}</p>
  </div>

  <ChevronsUpDownIcon class="ml-auto size-4 shrink-0 text-muted-foreground" />
{/if}
