import { defineConfig } from '@lingui/cli';

export default defineConfig({
  sourceLocale: 'en',
  locales: ['fr', 'en'],
  catalogs: [
    {
      path: '<rootDir>/src/modules/features/login/locales/{locale}/messages',
      include: ['src/modules/features/login/'],
    },
    {
      path: '<rootDir>/src/modules/features/user_settings/locales/{locale}/messages',
      include: ['src/modules/features/user_settings/'],
    },
  ],
  compileNamespace: 'default',
});