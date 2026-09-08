import { createRoot } from 'react-dom/client';
import './index.css';
import App from '@core/presentation/ui/App.tsx';
import { initializeAppLocale } from '@shared/presentation/utils/i18n.ts';

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('Root element not found');
}

// Catalogs are fetched per locale rather than bundled into the entry chunk, so
// the first render has to wait for them — otherwise the app paints a frame of
// untranslated message ids.
void initializeAppLocale().then(() => {
  createRoot(rootElement).render(<App />);
});
