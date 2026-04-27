import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/router/Core.router.tsx';
import { StrictMode } from 'react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { t } from '@lingui/core/macro';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider } from '@core/presentation/providers/Dependencies.provider.tsx';
import { messages as loginMessages } from '@/modules/features/login/locales/en/messages.ts';
import { messages as projectMessages } from '@/modules/features/project/locales/en/messages.ts';
import { messages as pipelineMessages } from '@/modules/features/pipeline/locales/en/messages.ts';
import { messages as marketplaceMessages } from '@/modules/features/marketplace/locales/en/messages.ts';
import { messages as organizationMessages } from '@/modules/features/organization/locales/en/messages.ts';
import { messages as userMessages } from '@/modules/features/user/locales/en/messages.ts';
import { messages as sharedMessages } from '@/locales/en/messages.ts';
import { messages as jobMessages } from '@/modules/features/jobs/locales/en/messages.ts';

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
});
i18n.activate('en');

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: error => {
      const scyllaError = error as ScyllaError;
      const code = scyllaError.getCode();

      if (code === 'UNAUTHENTICATED') {
        localStorage.removeItem('token');
        return;
      }

      scyllaError.log();
      const message = scyllaError.isNetworkError()
        ? t`Server unreachable`
        : scyllaError.message || t`An unexpected error occurred`;
      toast.error(message);
    },
  }),
  // Global mutation error handler — shows toast for all mutations.
  // Individual hooks should NOT add their own onError toast to avoid double-toasting.
  mutationCache: new MutationCache({
    onError: error => {
      const scyllaError = error as ScyllaError;
      scyllaError.log();
      const message = scyllaError.isNetworkError()
        ? t`Server unreachable`
        : scyllaError.message || t`Operation failed`;
      toast.error(message);
    },
  }),
});

function App() {
  return (
    <StrictMode>
      <I18nProvider i18n={i18n}>
        <QueryClientProvider client={queryClient}>
          <DependenciesProvider>
            <RouterProvider router={CoreRouter} />
            <Toaster />
          </DependenciesProvider>
        </QueryClientProvider>
      </I18nProvider>
    </StrictMode>
  );
}

export default App;
