import { mount } from 'svelte';
import './index.css';
import App from '@core/presentation/ui/App.svelte';
import { startRouter } from '@core/presentation/ui/router/core.router.ts';
import { initializeAppLocale } from '@shared/presentation/utils/i18n.ts';

const target = document.getElementById('root');

if (!target) {
  throw new Error('Root element not found');
}

void initializeAppLocale().then(() => {
  startRouter();
  mount(App, { target });
});
