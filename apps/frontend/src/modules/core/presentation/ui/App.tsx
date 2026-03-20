import { RouterProvider } from 'react-router-dom';
import { CoreRouter } from '@core/presentation/ui/CoreRouter.tsx';
import { StrictMode } from 'react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider } from '@core/presentation/providers/DependenciesProvider.tsx';
import { messages as loginMessages } from '@/modules/features/login/locales/en/messages.ts';
import { messages as userSettingsMessages } from '@/modules/features/user_settings/locales/en/messages.ts';
import { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';

i18n.load('en', {
  ...loginMessages,
  ...userSettingsMessages,
});
i18n.activate('en');

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: error => {
      (error as ScyllaError).log();
    },
  }),
  mutationCache: new MutationCache({
    onError: error => {
      (error as ScyllaError).log();
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
          </DependenciesProvider>
        </QueryClientProvider>
      </I18nProvider>
    </StrictMode>
  );
}

export default App;
