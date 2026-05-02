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
      path: '<rootDir>/src/modules/features/jobs/locales/{locale}/messages',
      include: ['src/modules/features/jobs/'],
    },
    {
      path: '<rootDir>/src/modules/features/user/locales/{locale}/messages',
      include: ['src/modules/features/user/'],
    },
    {
      path: '<rootDir>/src/modules/features/project/locales/{locale}/messages',
      include: ['src/modules/features/project/'],
    },
    {
      path: '<rootDir>/src/modules/features/pipeline/locales/{locale}/messages',
      include: ['src/modules/features/pipeline/'],
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
      path: '<rootDir>/src/modules/features/workers/locales/{locale}/messages',
      include: ['src/modules/features/workers/'],
    },
    {
      path: '<rootDir>/src/locales/{locale}/messages',
      include: ['src/modules/shared/', 'src/modules/layout/', 'src/modules/core/'],
    },
  ],
  compileNamespace: 'default',
});
