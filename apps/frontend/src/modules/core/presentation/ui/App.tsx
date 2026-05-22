import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/router/Core.router.tsx';
import { StrictMode } from 'react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { t } from '@lingui/core/macro';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider } from '@core/presentation/providers/Dependencies.provider.tsx';
import { CapabilitiesProvider } from '@core/presentation/providers/Capabilities.provider.tsx';
import { messages as loginMessages } from '@/modules/features/login/locales/en/messages.ts';
import { messages as projectMessages } from '@/modules/features/project/locales/en/messages.ts';
import { messages as pipelineMessages } from '@/modules/features/pipeline/locales/en/messages.ts';
import { messages as marketplaceMessages } from '@/modules/features/marketplace/locales/en/messages.ts';
import { messages as organizationMessages } from '@/modules/features/organization/locales/en/messages.ts';
import { messages as userMessages } from '@/modules/features/user/locales/en/messages.ts';
import { messages as sharedMessages } from '@/locales/en/messages.ts';
import { messages as jobMessages } from '@/modules/features/jobs/locales/en/messages.ts';
import { messages as appsMessages } from '@/modules/features/apps/locales/en/messages.ts';
import { messages as agentsMessages } from '@/modules/features/agents/locales/en/messages.ts';

import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Toaster } from '@shadcn/sonner.tsx';

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
  ...agentsMessages,
});
i18n.activate('en');

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

        const message = error.isNetworkError()
          ? t`Server unreachable`
          : error.message || t`An unexpected error occurred`;

        if (error.isNetworkError()) {
          localStorage.removeItem('token');
          window.location.href = '/login';
        }

        toast.error(message);
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

        const message = error.isNetworkError()
          ? t`Server unreachable`
          : error.message || t`Operation failed`;

        toast.error(message);
      } else {
        console.error('Mutation Error (Non-Scylla):', error);
      }
    },
  }),
});

function App() {
  return (
    <StrictMode>
      <I18nProvider i18n={i18n}>
        <QueryClientProvider client={queryClient}>
          <CapabilitiesProvider>
            <DependenciesProvider>
              <RouterProvider router={CoreRouter} />
              <Toaster />
            </DependenciesProvider>
          </CapabilitiesProvider>
        </QueryClientProvider>
      </I18nProvider>
    </StrictMode>
  );
}

export default App;
