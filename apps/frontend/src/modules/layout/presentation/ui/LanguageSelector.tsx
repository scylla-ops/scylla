import { LanguagesIcon } from 'lucide-react';
import { useLingui } from '@lingui/react/macro';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem, useSidebar } from '@shadcn/sidebar.tsx';
import { setAppLocale, type SupportedLocale } from '@shared/presentation/utils/i18n.ts';

interface LocaleOption {
  locale: SupportedLocale;
  label: string;
  short: string;
}

// Language names stay untranslated on purpose — a locale is always spelled in
// its own language so it stays recognizable whatever the active one is.
const LOCALE_OPTIONS: LocaleOption[] = [
  { locale: 'en', label: 'English', short: 'EN' },
  { locale: 'fr', label: 'Français', short: 'FR' },
];

export function LanguageSelector() {
  const { t, i18n } = useLingui();
  const { isMobile } = useSidebar();

  const currentLocale = (i18n.locale as SupportedLocale | undefined) ?? 'en';
  const current =
    LOCALE_OPTIONS.find(option => option.locale === currentLocale) ?? LOCALE_OPTIONS[0];

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              tooltip={`${t`Language`} · ${current.short}`}
              className='data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground'
            >
              <LanguagesIcon />
              <span className='flex-1 truncate'>{t`Language`}</span>
              <span className='text-xs text-muted-foreground'>{current.short}</span>
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className='min-w-40 rounded-lg border-border bg-background shadow-lg'
            side={isMobile ? 'bottom' : 'right'}
            align='end'
            sideOffset={4}
          >
            <DropdownMenuRadioGroup
              value={currentLocale}
              onValueChange={value => setAppLocale(value as SupportedLocale)}
            >
              {LOCALE_OPTIONS.map(option => (
                <DropdownMenuRadioItem key={option.locale} value={option.locale}>
                  {option.label}
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
