/// <reference types="vitest/config" />
import { defineConfig, type Plugin } from 'vite';
import { transformAsync } from '@babel/core';
import linguiMacroPlugin from '@lingui/babel-plugin-lingui-macro';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import tsconfigPaths from 'vite-tsconfig-paths';
import { lingui } from '@lingui/vite-plugin';

/**
 * Third-party code pinned to stable, separately cacheable chunks.
 *
 * The lazy `page` of each route already keeps it out of the entry chunk, but a
 * library shared by two lazy routes would otherwise be duplicated or hoisted
 * back into the entry. Splitting by library also means a dependency bump only
 * invalidates its own chunk instead of the whole bundle.
 *
 * An entry matches a package when it is the exact name (`svelte`), a scope
 * (`@codemirror` matches `@codemirror/view`), or a name prefix written with a
 * trailing dash (`d3-` would match `d3-scale`).
 */
const VENDOR_CHUNKS: Record<string, string[]> = {
  'vendor-svelte': ['svelte', 'esm-env', 'clsx'],
  'vendor-ui': [
    'bits-ui',
    '@lucide/svelte',
    'svelte-toolbelt',
    'runed',
    '@floating-ui',
    'tabbable',
    'svelte-sonner',
    'class-variance-authority',
    'tailwind-merge',
  ],
  'vendor-query': ['@tanstack'],
  'vendor-i18n': ['@lingui', 'messageformat-parser', '@messageformat'],
  'vendor-flow': ['@xyflow'],
  'vendor-codemirror': ['codemirror', '@codemirror', '@lezer'],
  'vendor-grpc': ['@protobuf-ts'],
};

/**
 * Compiles the Lingui macros (`msg`, `t`, `plural`) in TypeScript files.
 *
 * Only the files that import `@lingui/core/macro` go through Babel. Babel parses
 * TypeScript and does not remove the types: Vite does that after this plugin.
 */
const linguiMacros = (): Plugin => ({
  name: 'scylla:lingui-macros',
  enforce: 'pre',
  async transform(code, id) {
    const path = id.split('?')[0];
    if (!/\.(ts|js)$/.test(path) || path.includes('/node_modules/')) return null;
    if (!code.includes('@lingui/core/macro')) return null;

    const result = await transformAsync(code, {
      filename: path,
      babelrc: false,
      configFile: false,
      sourceMaps: true,
      parserOpts: { plugins: ['typescript'] },
      plugins: [linguiMacroPlugin],
    });

    return result?.code ? { code: result.code, map: result.map } : null;
  },
});

/** `…/node_modules/@scope/name/dist/x.js` -> `@scope/name`. */
const packageNameOf = (id: string): string => {
  const withinModules = id.split('node_modules/').pop() ?? '';
  const segments = withinModules.split('/');
  return segments[0].startsWith('@') ? `${segments[0]}/${segments[1]}` : segments[0];
};

const matches = (packageName: string, entry: string): boolean =>
  packageName === entry ||
  packageName.startsWith(`${entry}/`) ||
  (entry.endsWith('-') && packageName.startsWith(entry));

export default defineConfig({
  // Svelte ships a server build and a client one, and picks by export condition.
  // Under Vitest the default resolution lands on the server build, where `mount`
  // throws `lifecycle_function_unavailable`. Scoped to the test run on purpose:
  // forcing `browser` for the production build would change how every dependency
  // resolves, not just Svelte.
  resolve: process.env.VITEST ? { conditions: ['browser'] } : {},
  plugins: [
    lingui(),
    linguiMacros(),
    svelte(),
    tailwindcss(),
    // `loose` is what makes `@platform/…` resolve from a `.svelte` file: by
    // default the plugin only rewrites imports coming from a JS/TS importer, so
    // every alias inside a component silently failed to resolve.
    tsconfigPaths({ loose: true }),
  ],
  optimizeDeps: {
    exclude: [
      '@lucide/svelte',
      'bits-ui',
      '@tanstack/svelte-query',
      '@tanstack/svelte-table',
      '@xyflow/svelte',
      'svelte-sonner',
    ],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          // One chunk per locale rather than one per module's catalog: the
          // catalogs are loaded together for the active locale, so ~17 tiny
          // requests would buy nothing.
          const locale = /\/locales\/([^/]+)\/messages\.ts$/.exec(id)?.[1];
          if (locale) return `locale-${locale}`;

          if (!id.includes('node_modules')) return undefined;

          const packageName = packageNameOf(id);

          for (const [chunk, packages] of Object.entries(VENDOR_CHUNKS)) {
            if (packages.some(entry => matches(packageName, entry))) return chunk;
          }

          return undefined;
        },
      },
    },
  },
  test: {
    // Default for the UI tests. Mappers, utils and domain tests opt out with a
    // `// @vitest-environment node` docblock — building a jsdom for a pure
    // function was ~40% of the suite's wall time.
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    css: false,
    coverage: {
      provider: 'v8',
      // `include` is what makes untested files count: everything matching is
      // reported at 0% rather than being absent, which is the difference
      // between a real number and one that flatters itself.
      include: ['src/modules/**/*.{ts,svelte}'],
      exclude: [
        // Machine output: generated proto clients and compiled Lingui catalogs.
        'src/generated/**',
        '**/locales/**',
        '**/*.test.ts',
        // Test scaffolding too: a `*.fixture.svelte` exists to pin a generic or
        // to compose parts a raw snippet cannot build, and it is rendered only
        // by the test beside it.
        '**/*.fixture.{ts,svelte}',
        // Vendored shadcn primitives — upstream code we don't own.
        '**/shadcn/**',
        // Barrels and module declarations are re-exports and wiring: covering
        // them measures nothing, and `*.module.ts` pulls a feature's gRPC
        // client in just by being imported.
        '**/index.ts',
        '**/*.module.ts',
      ],
      reporter: ['text-summary', 'html'],
      // A ratchet, not a target: these are the levels reached today, so the
      // number can only go up. Raise them when a batch of tests lands; never
      // lower them to make a red run green.
      thresholds: {
        statements: 77,
        branches: 70,
        functions: 74,
        lines: 77,
      },
    },
  },
});
