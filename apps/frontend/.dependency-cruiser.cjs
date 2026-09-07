/**
 * Architecture rules for src/modules, as a machine check of the contract in
 * CLAUDE.md. These rules describe the architecture we are converging on, so
 * some of them still report violations today — CI runs this in reporting mode
 * until the module graph is cycle-free, then it becomes a gate.
 *
 * `pnpm depcruise`       -> validate, human-readable
 * `pnpm depcruise:graph` -> SVG of the module graph (needs graphviz)
 *
 * @type {import('dependency-cruiser').IConfiguration}
 */
module.exports = {
  forbidden: [
    {
      name: 'no-circular',
      comment:
        'A cycle means the two modules are really one. It also blocks route-level code ' +
        'splitting and makes the graph impossible to reason about.',
      severity: 'error',
      from: {},
      to: {
        circular: true,
        // A cycle that runs through a dynamic import is not a cycle at runtime —
        // it is a chunk boundary. Modules declare their pages with react-router
        // `lazy`, so `x.module.ts -> (lazy) SomePage -> a hook -> use-x-domain.ts
        // -> (type only) x.module.ts` is expected and is precisely what makes the
        // page a separate chunk. Only flag cycles where every edge is static.
        viaOnly: { dependencyTypesNot: ['dynamic-import'] },
      },
    },

    {
      name: 'shared-is-generic',
      comment:
        'shared/ holds reusable UI and utils with no business meaning. The moment it ' +
        'knows about a feature it stops being reusable and becomes a cycle.',
      severity: 'error',
      from: { path: '^src/modules/shared/' },
      to: { path: '^src/modules/(features|core|layout|platform|app)/' },
    },

    {
      name: 'platform-knows-no-feature',
      comment:
        'platform/ sits below features so every feature can depend on it. It may never ' +
        'depend back on one.',
      severity: 'error',
      from: { path: '^src/modules/platform/' },
      to: { path: '^src/modules/(features|layout|app)/' },
    },

    {
      name: 'domain-is-pure',
      comment:
        'domain/ is pure business logic: no React, no gRPC, no generated proto, no ' +
        'query/state library. Framework types belong in infrastructure or presentation.',
      severity: 'error',
      from: { path: '^src/modules/.+/domain/' },
      to: {
        path: [
          '^src/generated/',
          '^node_modules/(react|react-dom|zustand|@tanstack|@lingui|@protobuf-ts|lucide-react|react-router)',
        ],
      },
    },

    {
      name: 'domain-does-not-know-infrastructure',
      comment:
        'The dependency inversion the layering exists for: domain declares repository ' +
        'interfaces, infrastructure implements them, never the reverse.',
      severity: 'error',
      from: { path: '^src/modules/(.+)/domain/' },
      to: { path: '^src/modules/.+/infrastructure/' },
    },

    {
      name: 'no-feature-imports-the-shell',
      comment:
        'The shell (layout/, app/, core/ router) composes features. A feature reaching ' +
        'back into it inverts the composition root.',
      severity: 'error',
      from: { path: '^src/modules/features/' },
      to: { path: '^src/modules/(layout|app)/' },
    },

    {
      name: 'not-to-dev-dep',
      comment: 'Shipped code must not import a devDependency.',
      severity: 'error',
      // `.d.ts` files are excluded: src/vite-env.d.ts legitimately references
      // vite/client, which is types-only and never reaches the bundle.
      from: { path: '^src/', pathNot: ['[.](spec|test)[.](ts|tsx)$', '[.]d[.]ts$'] },
      to: { dependencyTypes: ['npm-dev'], dependencyTypesNot: ['type-only'] },
    },

    {
      name: 'no-orphans',
      comment:
        'A module nothing imports is usually dead code left behind by a refactor. ' +
        'Config, type declarations and entry points are excluded.',
      severity: 'warn',
      from: {
        orphan: true,
        pathNot: [
          '(^|/)[.][^/]+[.](js|cjs|mjs|ts|json)$',
          '[.]d[.]ts$',
          '(^|/)tsconfig[.]json$',
          '(^|/)(vite|eslint|lingui|postcss)[.]config[.][^/]+$',
          '^src/main[.]tsx$',
          '^src/generated/',
        ],
      },
      to: {},
    },
  ],

  options: {
    doNotFollow: { path: 'node_modules' },

    // Generated proto clients and compiled Lingui catalogs are machine output —
    // they have their own shape and are not ours to police.
    exclude: { path: ['^src/generated/', '/locales/'] },

    // Resolves the @/ @core/ @shared/ @shadcn/ aliases the codebase imports by.
    tsConfig: { fileName: 'tsconfig.app.json' },

    // Follow type-only imports too: `import type { UserEntity }` across a module
    // boundary is still a coupling, and still a cycle.
    tsPreCompilationDeps: true,

    enhancedResolveOptions: {
      exportsFields: ['exports'],
      conditionNames: ['import', 'require', 'node', 'default', 'types'],
      extensions: ['.js', '.jsx', '.ts', '.tsx'],
      mainFields: ['module', 'main', 'types', 'typings'],
    },

    reporterOptions: {
      dot: { collapsePattern: '^src/modules/(features/[^/]+|[^/]+)' },
      archi: { collapsePattern: '^src/modules/(features/[^/]+|[^/]+)' },
      text: { highlightFocused: true },
    },
  },
};
