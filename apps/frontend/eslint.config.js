import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { globalIgnores } from 'eslint/config'

export default tseslint.config([
  // ── Ignored paths ────────────────────────────────────────────────────────────
  globalIgnores([
    'dist',
    // Auto-generated protobuf files: contain intentional `any` and @ts-nocheck
    'src/generated/**',
    // Lingui compiled message catalogs
    'src/**/locales/**',
  ]),

  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      // Type-aware rules: catches floating promises, misused promises, etc.
      // Requires parserOptions.projectService below.
      tseslint.configs.recommendedTypeChecked,
      reactHooks.configs['recommended-latest'],
      reactRefresh.configs.vite,
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

      // ── React Fast Refresh ────────────────────────────────────────────────────
      // shadcn components export CVA configs alongside components — this is the
      // expected shadcn pattern and does not break Fast Refresh in practice.
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
  },

  // ── shadcn UI library files ───────────────────────────────────────────────────
  // These are auto-generated / copy-pasted from shadcn and follow their own
  // conventions. We relax a few rules that would otherwise fire on every update.
  {
    files: ['src/modules/shared/presentation/ui/shadcn/**'],
    rules: {
      '@typescript-eslint/restrict-template-expressions': 'off',
      '@typescript-eslint/no-unnecessary-type-assertion': 'off',
      'react-refresh/only-export-components': 'off',
    },
  },
])



