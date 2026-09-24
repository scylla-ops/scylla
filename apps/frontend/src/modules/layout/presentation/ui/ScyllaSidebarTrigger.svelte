<script lang="ts">
  import ArrowLeftToLineIcon from '@lucide/svelte/icons/arrow-left-to-line';
  import { Button, getSidebar } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { layoutMessages } from './layout.messages.ts';

  const sidebar = getSidebar();
</script>

<Button
  data-sidebar="trigger"
  data-slot="sidebar-trigger"
  variant="ghost"
  size="icon"
  class="size-7"
  onclick={() => sidebar.toggle()}
>
  <ArrowLeftToLineIcon
    class={cn('size-4 transition-transform duration-300', !sidebar.open && 'rotate-180')}
  />
  <span class="sr-only">{t(layoutMessages.toggleSidebar)}</span>
</Button>
