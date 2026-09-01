import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/router/Core.router.tsx';
import { StrictMode } from 'react';
import { I18nProvider } from '@lingui/react';
import { useLingui } from '@lingui/react/macro';
import { i18n } from '@lingui/core';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider } from '@core/presentation/providers/Dependencies.provider.tsx';
import { ThemeProvider, useTheme } from 'next-themes';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/modules/shared/presentation/ui/shadcn/button.tsx';
import type { Messages } from '@lingui/core';

import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Toaster } from '@shadcn/sonner.tsx';
import { initializeAppLocale } from '@shared/presentation/utils/i18n.ts';

// Every compiled catalog under src/modules is picked up by convention, so a new
// module (or a new locale) is live as soon as lingui.config.js knows about it.
// Listing them by hand is how `core` ended up extracted but never loaded.
const catalogs = import.meta.glob<{ messages: Messages }>('../../../**/locales/*/messages.ts', {
  eager: true,
});

const LOCALE_FROM_PATH = /\/locales\/([^/]+)\/messages\.ts$/;

const messagesByLocale = Object.entries(catalogs).reduce<Record<string, Messages>>(
  (acc, [path, module]) => {
    const locale = LOCALE_FROM_PATH.exec(path)?.[1];
    if (locale) acc[locale] = { ...acc[locale], ...module.messages };
    return acc;
  },
  {},
);

for (const [locale, messages] of Object.entries(messagesByLocale)) {
  i18n.load(locale, messages);
}

initializeAppLocale();

//todo: maybe in production, console error only network or non scylla error ?
//todo: domain errors should be only be toasted by module itself
// for the beta its okay, because it allow us to be able to collect more easily the problems encountered by users

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: error => {
      if (error instanceof ScyllaError) {
        const code = error.getCode();

        if (code === 'UNAUTHENTICATED') {
          localStorage.removeItem('token');
          window.location.href = '/login';
          return;
        }

        error.log();

        if (error.isNetworkError()) {
          localStorage.removeItem('token');
          window.location.href = '/login';
        }

        toast.error(error.userMessage());
      } else {
        console.error('Non-Scylla Error:', error);
      }
    },
  }),
  // Global mutation error handler — shows toast for all mutations.
  // Individual hooks should NOT add their own onError toast to avoid double-toasting.
  mutationCache: new MutationCache({
    onError: error => {
      if (error instanceof ScyllaError) {
        const code = error.getCode();

        if (code === 'UNAUTHENTICATED') {
          localStorage.removeItem('token');
          window.location.href = '/login';
          return;
        }

        error.log();

        toast.error(error.userMessage());
      } else {
        console.error('Mutation Error (Non-Scylla):', error);
      }
    },
  }),
});

function ThemeToggle() {
  const { t } = useLingui();
  const { resolvedTheme, setTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  return (
    <Button
      type='button'
      variant='outline'
      size='icon'
      onClick={() => setTheme(isDark ? 'light' : 'dark')}
      aria-label={t`Toggle dark mode`}
      className='fixed right-4 top-4 z-50 h-10 w-10 rounded-full border-border bg-background/90 shadow-sm backdrop-blur'
    >
      {isDark ? (
        <Sun className='size-4 text-amber-500' />
      ) : (
        <Moon className='size-4 text-slate-600 dark:text-slate-300' />
      )}
    </Button>
  );
}

function App() {
  return (
    <StrictMode>
      <ThemeProvider
        attribute='class'
        defaultTheme='dark'
        enableSystem={false}
        storageKey='scylla-theme'
        disableTransitionOnChange
      >
        <I18nProvider i18n={i18n}>
          <QueryClientProvider client={queryClient}>
            <DependenciesProvider>
              <ThemeToggle />
              <RouterProvider router={CoreRouter} />
              <Toaster />
            </DependenciesProvider>
          </QueryClientProvider>
        </I18nProvider>
      </ThemeProvider>
    </StrictMode>
  );
}

export default App;
