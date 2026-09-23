<script lang="ts">
  import MoonIcon from '@lucide/svelte/icons/moon';
  import SunIcon from '@lucide/svelte/icons/sun';
  import { Button } from '@shadcn-svelte';
  import { getTheme, setTheme } from '@shared/presentation/stores/theme.store.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { coreMessages } from './router/core.messages.ts';
</script>

<Button
  type="button"
  variant="outline"
  size="icon"
  onclick={() => setTheme(getTheme() === 'dark' ? 'light' : 'dark')}
  aria-label={t(coreMessages.toggleDarkMode)}
  class="fixed top-4 right-4 z-50 h-10 w-10 rounded-full border-border bg-background/90 shadow-sm backdrop-blur"
>
  <SunIcon class="hidden size-4 text-amber-500 dark:block" />
  <MoonIcon class="size-4 text-slate-600 dark:hidden dark:text-slate-300" />
</Button>
