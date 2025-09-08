import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import { DependenciesProvider } from '@/modules/core/presentation/providers/DependenciesProvider.tsx';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { messages as coreMessages } from './modules/core/locales/en/messages';
import { messages as loginMessages } from './modules/login/locales/en/messages';

i18n.load('en', {
  ...coreMessages,
  ...loginMessages,
});
i18n.activate('en');

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <I18nProvider i18n={i18n}>
      <DependenciesProvider>
        <App />
      </DependenciesProvider>
    </I18nProvider>
  </StrictMode>,
);
