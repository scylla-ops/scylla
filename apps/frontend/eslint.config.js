import js from '@eslint/js'
import globals from 'globals'
import tseslint from 'typescript-eslint'
import svelte from 'eslint-plugin-svelte'
import svelteParser from 'svelte-eslint-parser'
import { globalIgnores } from 'eslint/config'

export default tseslint.config([
  // ── Ignored paths ────────────────────────────────────────────────────────────
  globalIgnores([
    'dist',
    // Auto-generated protobuf files: contain intentional `any` and @ts-nocheck
    'src/generated/**',
    // Lingui compiled message catalogs
    'src/**/locales/**',
    // v8 coverage output — the HTML reporter ships its own vendored scripts
    'coverage/**',
  ]),

  {
    files: ['**/*.ts'],
    extends: [
      js.configs.recommended,
      // Type-aware rules: catches floating promises, misused promises, etc.
      // Requires parserOptions.projectService below.
      tseslint.configs.recommendedTypeChecked,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        // Automatically resolves the right tsconfig per file (v8+ feature).
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // ── Unused code ───────────────────────────────────────────────────────────
      // Prefix with _ to intentionally suppress (e.g. `_unusedParam`).
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
        },
      ],

      // ── Async correctness ─────────────────────────────────────────────────────
      // Prevent fire-and-forget promises (very common source of silent bugs).
      '@typescript-eslint/no-floating-promises': 'error',
      // Prevent passing async callbacks where void is expected (e.g. onClick).
      '@typescript-eslint/no-misused-promises': [
        'error',
        { checksVoidReturn: { attributes: false } },
      ],

      // ── Import hygiene ────────────────────────────────────────────────────────
      // Enforce `import type` for type-only imports (required by verbatimModuleSyntax).
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],

      // ── Disable overly noisy "unsafe" rules ───────────────────────────────────
      // These fire heavily on anything that interacts with the generated protobuf
      // layer (which uses internal `any` types). Structural safety is enforced by
      // TypeScript strict mode in tsconfig instead.
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      // Enum comparisons between domain ↔ gRPC enums (same values, different types).
      '@typescript-eslint/no-unsafe-enum-comparison': 'off',

      // Static mapper methods (GrpcXxxMapper.toDomain etc.) are pure functions that
      // never access `this` — treating them as unbound is a false positive.
      '@typescript-eslint/unbound-method': ['error', { ignoreStatic: true }],
    },
  },

  // ── Tests ──────────────────────────────────────────────────────────────────────
  // `expect(repository.method)` reads a mock off an object and never calls it
  // unbound. `unbound-method` flags every such assertion, with no true positive.
  {
    files: ['**/*.test.ts'],
    rules: {
      '@typescript-eslint/unbound-method': 'off',
    },
  },

  // ── The Svelte query bindings come from @platform/query ───────────────────────
  // `createQuery` from `@tanstack/svelte-query` reads its client from Svelte
  // context. The re-export in `@platform/query` binds the app's client; the two
  // are indistinguishable at the call site, so the wrong import fails at runtime.
  {
    files: ['src/modules/**/*.{ts,svelte}'],
    ignores: ['src/modules/platform/query/**'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tanstack/svelte-query',
              message:
                'Import createQuery / createMutation from @platform/query — they carry the ' +
                "app's QueryClient.",
            },
          ],
        },
      ],
    },
  },

  // ── Svelte ────────────────────────────────────────────────────────────────────
  // `.svelte` files are invisible to `tsc -b`; `pnpm typecheck` runs svelte-check
  // after it for exactly that reason. Here we only need the parser and the
  // plugin's own rules.
  ...svelte.configs.recommended,
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    plugins: { '@typescript-eslint': tseslint.plugin },
    languageOptions: {
      parser: svelteParser,
      globals: globals.browser,
      parserOptions: {
        parser: tseslint.parser,
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: ['.svelte'],
      },
    },
    rules: {
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
    },
  },

])



