import { i18n } from '@lingui/core';

export type SupportedLocale = 'en' | 'fr';

const STORAGE_KEY = 'scylla-locale';

function isSupportedLocale(locale: string | null | undefined): locale is SupportedLocale {
  return locale === 'en' || locale === 'fr';
}

export function getStoredLocale(): SupportedLocale {
  if (typeof window === 'undefined') {
    return 'en';
  }

  const storedLocale = window.localStorage.getItem(STORAGE_KEY);
  if (isSupportedLocale(storedLocale)) {
    return storedLocale;
  }

  const browserLocale = window.navigator.language.split('-')[0];
  return browserLocale === 'fr' ? 'fr' : 'en';
}

export function setAppLocale(locale: SupportedLocale): void {
  i18n.activate(locale);

  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale;
  }
}

export function initializeAppLocale(): SupportedLocale {
  const locale = getStoredLocale();
  setAppLocale(locale);
  return locale;
}

export function getCurrentLocale(): SupportedLocale {
  return (i18n.locale as SupportedLocale | undefined) ?? 'en';
}
