/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import tailwindcss from '@tailwindcss/vite';
import tsconfigPaths from 'vite-tsconfig-paths';
import { lingui } from '@lingui/vite-plugin';

/**
 * Third-party code pinned to stable, separately cacheable chunks.
 *
 * Route-level `lazy` already keeps each page out of the entry chunk, but a
 * library shared by two lazy routes would otherwise be duplicated or hoisted
 * back into the entry. Splitting by library also means a dependency bump only
 * invalidates its own chunk instead of the whole bundle.
 *
 * An entry matches a package when it is the exact name (`react`), a scope
 * (`@radix-ui` matches `@radix-ui/react-dialog`), or a name prefix written with
 * a trailing dash (`d3-` matches `d3-scale`). Substring matching would be wrong
 * here — plain `react` would otherwise swallow `reactflow` and `lucide-react`.
 */
const VENDOR_CHUNKS: Record<string, string[]> = {
  'vendor-react': ['react', 'react-dom', 'react-router', 'react-router-dom', 'scheduler'],
  'vendor-ui': [
    '@radix-ui',
    'radix-ui',
    'lucide-react',
    'sonner',
    'next-themes',
    'class-variance-authority',
    'tailwind-merge',
    'clsx',
  ],
  'vendor-query': ['@tanstack'],
  'vendor-i18n': ['@lingui', 'messageformat-parser', '@messageformat'],
  'vendor-flow': ['reactflow', '@reactflow'],
  'vendor-codemirror': ['@uiw', 'codemirror', '@codemirror', '@lezer'],
  'vendor-charts': ['recharts', 'victory-vendor', 'd3-', 'internmap', 'decimal.js-light'],
  'vendor-motion': ['framer-motion', 'motion-dom', 'motion-utils'],
  'vendor-grpc': ['@protobuf-ts'],
};

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
  plugins: [
    lingui(),
    react({
      plugins: [['@lingui/swc-plugin', {}]],
    }),
    tailwindcss(),
    tsconfigPaths(),
  ],
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
      include: ['src/modules/**/*.{ts,tsx}'],
      exclude: [
        // Machine output: generated proto clients and compiled Lingui catalogs.
        'src/generated/**',
        '**/locales/**',
        '**/*.test.{ts,tsx}',
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
        statements: 64,
        branches: 64,
        functions: 58,
        lines: 64,
      },
    },
  },
});
