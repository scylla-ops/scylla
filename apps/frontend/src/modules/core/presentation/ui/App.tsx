import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/router/Core.router.tsx';
import { StrictMode } from 'react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider } from '@core/presentation/providers/Dependencies.provider.tsx';
import { ThemeProvider, useTheme } from 'next-themes';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/modules/shared/presentation/ui/shadcn/button.tsx';
import { messages as loginMessages } from '@/modules/features/login/locales/en/messages.ts';
import { messages as projectMessages } from '@/modules/features/project/locales/en/messages.ts';
import { messages as pipelineMessages } from '@/modules/features/pipeline/locales/en/messages.ts';
import { messages as marketplaceMessages } from '@/modules/features/marketplace/locales/en/messages.ts';
import { messages as organizationMessages } from '@/modules/features/organization/locales/en/messages.ts';
import { messages as userMessages } from '@/modules/features/user/locales/en/messages.ts';
import { messages as sharedMessages } from '@/modules/shared/locales/en/messages.ts';
import { messages as layoutMessages } from '@/modules/layout/locales/en/messages.ts';
import { messages as jobMessages } from '@/modules/features/jobs/locales/en/messages.ts';
import { messages as appsMessages } from '@/modules/features/apps/locales/en/messages.ts';
import { messages as agentsMessages } from '@/modules/features/agents/locales/en/messages.ts';
import { messages as secretMessages } from '@/modules/features/secret/locales/en/messages.ts';
import { messages as triggersMessages } from '@/modules/features/triggers/locales/en/messages.ts';
import { messages as permissionMessages } from '@/modules/features/permission/locales/en/messages.ts';
import { messages as loginFrMessages } from '@/modules/features/login/locales/fr/messages.ts';
import { messages as projectFrMessages } from '@/modules/features/project/locales/fr/messages.ts';
import { messages as pipelineFrMessages } from '@/modules/features/pipeline/locales/fr/messages.ts';
import { messages as marketplaceFrMessages } from '@/modules/features/marketplace/locales/fr/messages.ts';
import { messages as organizationFrMessages } from '@/modules/features/organization/locales/fr/messages.ts';
import { messages as userFrMessages } from '@/modules/features/user/locales/fr/messages.ts';
import { messages as sharedFrMessages } from '@/modules/shared/locales/fr/messages.ts';
import { messages as layoutFrMessages } from '@/modules/layout/locales/fr/messages.ts';
import { messages as jobFrMessages } from '@/modules/features/jobs/locales/fr/messages.ts';
import { messages as appsFrMessages } from '@/modules/features/apps/locales/fr/messages.ts';
import { messages as agentsFrMessages } from '@/modules/features/agents/locales/fr/messages.ts';
import { messages as secretFrMessages } from '@/modules/features/secret/locales/fr/messages.ts';
import { messages as triggersFrMessages } from '@/modules/features/triggers/locales/fr/messages.ts';
import { messages as permissionFrMessages } from '@/modules/features/permission/locales/fr/messages.ts';

import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Toaster } from '@shadcn/sonner.tsx';
import { initializeAppLocale } from '@shared/presentation/utils/i18n.ts';

i18n.load('en', {
  ...loginMessages,
  ...userMessages,
  ...projectMessages,
  ...pipelineMessages,
  ...marketplaceMessages,
  ...organizationMessages,
  ...sharedMessages,
  ...jobMessages,
  ...appsMessages,
  ...layoutMessages,
  ...agentsMessages,
  ...secretMessages,
  ...triggersMessages,
  ...permissionMessages,
});
i18n.load('fr', {
  ...loginFrMessages,
  ...userFrMessages,
  ...projectFrMessages,
  ...pipelineFrMessages,
  ...marketplaceFrMessages,
  ...organizationFrMessages,
  ...sharedFrMessages,
  ...jobFrMessages,
  ...appsFrMessages,
  ...layoutFrMessages,
  ...agentsFrMessages,
  ...secretFrMessages,
  ...triggersFrMessages,
  ...permissionFrMessages,
});
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
  const { resolvedTheme, setTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  return (
    <Button
      type='button'
      variant='outline'
      size='icon'
      onClick={() => setTheme(isDark ? 'light' : 'dark')}
      aria-label='Toggle dark mode'
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
