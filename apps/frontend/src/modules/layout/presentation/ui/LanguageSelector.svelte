<script lang="ts">
  import LanguagesIcon from '@lucide/svelte/icons/languages';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuTrigger,
    getSidebar,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from '@shadcn';
  import { setAppLocale, type SupportedLocale } from '@shared/presentation/utils/i18n.ts';
  import { activeLocale, t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { layoutMessages } from './layout.messages.ts';

  const LOCALE_OPTIONS: { locale: SupportedLocale; label: string; short: string }[] = [
    { locale: 'en', label: 'English', short: 'EN' },
    { locale: 'fr', label: 'Français', short: 'FR' },
  ];

  const sidebar = getSidebar();

  const current = $derived(
    LOCALE_OPTIONS.find(option => option.locale === activeLocale()) ?? LOCALE_OPTIONS[0],
  );
</script>

<SidebarMenu>
  <SidebarMenuItem>
    <DropdownMenu>
      <DropdownMenuTrigger>
        {#snippet child({ props })}
          <SidebarMenuButton
            {...props}
            tooltip={`${t(layoutMessages.language)} · ${current.short}`}
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <LanguagesIcon />
            <span class="flex-1 truncate">{t(layoutMessages.language)}</span>
            <span class="text-xs text-muted-foreground">{current.short}</span>
          </SidebarMenuButton>
        {/snippet}
      </DropdownMenuTrigger>
      <DropdownMenuContent
        class="min-w-40 rounded-lg border-border bg-background shadow-lg"
        side={sidebar.isMobile ? 'bottom' : 'right'}
        align="end"
        sideOffset={4}
      >
        <DropdownMenuRadioGroup
          value={current.locale}
          onValueChange={value => void setAppLocale(value as SupportedLocale)}
        >
          {#each LOCALE_OPTIONS as option (option.locale)}
            <DropdownMenuRadioItem value={option.locale}>{option.label}</DropdownMenuRadioItem>
          {/each}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  </SidebarMenuItem>
</SidebarMenu>
