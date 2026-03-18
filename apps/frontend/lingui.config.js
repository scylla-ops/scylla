import { defineConfig } from '@lingui/cli';

export default defineConfig({
  sourceLocale: 'en',
  locales: ['fr', 'en'],
  catalogs: [
    {
      path: '<rootDir>/src/modules/core/locales/{locale}/messages',
      include: ['src/modules/features/core/'],
    },
    {
      path: '<rootDir>/src/modules/login/locales/{locale}/messages',
      include: ['src/modules/features/login/'],
    },
  ],
  compileNamespace: 'default',
});
