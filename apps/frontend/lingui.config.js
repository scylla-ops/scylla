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
    {
      path: '<rootDir>/src/modules/features/project/locales/{locale}/messages',
      include: ['src/modules/features/project/'],
    },
    {
      path: '<rootDir>/src/modules/features/pipeline-dashboard/locales/{locale}/messages',
      include: ['src/modules/features/pipeline-dashboard/'],
    },
    {
      path: '<rootDir>/src/modules/features/pipeline-creation/locales/{locale}/messages',
      include: ['src/modules/features/pipeline-creation/'],
    },
    {
      path: '<rootDir>/src/modules/features/marketplace/locales/{locale}/messages',
      include: ['src/modules/features/marketplace/'],
    },
    {
      path: '<rootDir>/src/modules/features/organization/locales/{locale}/messages',
      include: ['src/modules/features/organization/'],
    },
    {
      path: '<rootDir>/src/locales/{locale}/messages',
      include: ['src/modules/shared/', 'src/modules/layout/', 'src/modules/core/'],
    },
  ],
  compileNamespace: 'default',
});
