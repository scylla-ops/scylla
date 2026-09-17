import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/router/Core.router.tsx';
import { I18nProvider } from '@lingui/react';
import { useLingui } from '@lingui/react/macro';
import { i18n } from '@lingui/core';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from '@platform/query';
import { DependenciesProvider } from '@platform/di';
import { dependencies } from '@core/di/registry.ts';
import { useTheme } from '@shared/presentation/hooks/use-theme.ts';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/modules/shared/presentation/ui/shadcn/button.tsx';

import { Toaster } from '@shadcn/sonner.tsx';

// Catalogs are loaded and the locale activated in `main.tsx`, before the first
// render — see `initializeAppLocale`.

function ThemeToggle() {
  const { t } = useLingui();
  const { theme, setTheme } = useTheme();
  const isDark = theme === 'dark';

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

// No theme provider: `index.html` applies the class before the first paint and
// the theme store owns it from there, so nothing about the theme needs React
// context — see `shared/presentation/stores/theme.store.ts`.
function App() {
  return (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={dependencies}>
          <ThemeToggle />
          <RouterProvider router={CoreRouter} />
          <Toaster />
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
}

export default App;
