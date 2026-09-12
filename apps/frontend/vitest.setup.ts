import '@testing-library/jest-dom/vitest';
import { i18n } from '@lingui/core';

// The app itself loads compiled catalogs and activates a locale in App.tsx
// (see modules/core/presentation/ui/App.tsx). Tests don't render that entry
// point, so `t`/`Trans` need at least a loaded+activated locale to resolve —
// an empty catalog is enough: lingui falls back to the message id, which
// happens to be the source English text for every macro call in this
// codebase, and loading (even empty) silences its "not loaded" warning.
i18n.load('en', {});
i18n.activate('en');
