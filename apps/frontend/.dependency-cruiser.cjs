/**
 * Architecture rules for src/modules, as a machine check of the contract in
 * CLAUDE.md. Every rule below is a hard gate in CI (`.github/workflows/frontend.yml`);
 * none of them is aspirational.
 *
 * They come in two families:
 *  - *direction*  — who may depend on whom (`shared-is-generic`,
 *    `platform-knows-no-feature`, `domain-is-pure`, …);
 *  - *surface*    — how they may reach it. A module is reachable only through
 *    its `index.ts`, so its internals stay free to move (`feature-api-only`,
 *    `platform-api-only`, …).
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
      name: 'feature-api-only',
      comment:
        "A feature is reachable only through its `index.ts`. Reaching into another " +
        "feature's `domain/`, `presentation/` or `infrastructure/` makes every file in " +
        'it public, so no internal can be moved without breaking someone else — and it ' +
        'is how the module graph became a single strongly connected component before.',
      severity: 'error',
      from: { path: '^src/modules/features/([^/]+)/' },
      to: {
        path: '^src/modules/features/[^/]+/.+',
        pathNot: [
          // its own internals
          '^src/modules/features/$1/',
          // another feature's public API
          '^src/modules/features/[^/]+/index[.]ts$',
        ],
      },
    },

    {
      name: 'shell-uses-feature-api',
      comment:
        'The shell composes features, so it may import them — but through the same public ' +
        'API everyone else uses. The one extra door is `<feature>.module.ts`, which the ' +
        'registry imports on purpose (see `module-declaration-is-private`).',
      severity: 'error',
      from: { path: '^src/modules/(core|layout)/' },
      to: {
        path: '^src/modules/features/[^/]+/.+',
        pathNot: [
          '^src/modules/features/[^/]+/index[.]ts$',
          '^src/modules/features/[^/]+/[^/]+[.]module[.]ts$',
        ],
      },
    },

    {
      name: 'platform-api-only',
      comment:
        'Same contract as features, for the capabilities below them: import `@platform/authz`, ' +
        'not `@platform/authz/presentation/stores/…`.',
      severity: 'error',
      from: { path: '^src/modules/(features|core|layout)/' },
      to: {
        path: '^src/modules/platform/[^/]+/.+',
        pathNot: '^src/modules/platform/[^/]+/index[.]ts$',
      },
    },

    {
      name: 'platform-capability-api-only',
      comment:
        'Platform capabilities are modules too: `routing` reaches `authz` through its ' +
        'public API, not through its internals.',
      severity: 'error',
      from: { path: '^src/modules/platform/([^/]+)/' },
      to: {
        path: '^src/modules/platform/[^/]+/.+',
        pathNot: ['^src/modules/platform/$1/', '^src/modules/platform/[^/]+/index[.]ts$'],
      },
    },

    {
      name: 'module-declaration-is-private',
      comment:
        '`<feature>.module.ts` instantiates the module\'s infrastructure at import time, so ' +
        'importing it eagerly pulls that feature\'s gRPC client into your chunk. Only the ' +
        'registry (`core/di/registry.ts`) and the feature itself may.',
      severity: 'error',
      from: { path: '^src/modules/features/([^/]+)/' },
      to: {
        path: '^src/modules/features/[^/]+/[^/]+[.]module[.]ts$',
        pathNot: '^src/modules/features/$1/',
      },
    },

    {
      name: 'domain-accessor-is-private',
      comment:
        '`use-<feature>-domain.ts` is where a feature pins the type of its own injection. ' +
        'Another module calling it would query that feature\'s repository directly — ' +
        'bypassing its hooks and forking the query cache into two keys for one resource. ' +
        'Consume the feature\'s hooks through its `index.ts` instead.',
      severity: 'error',
      from: { path: '^src/modules/features/([^/]+)/' },
      to: {
        path: '^src/modules/features/[^/]+/presentation/hooks/use-[^/]+-domain[.]ts$',
        pathNot: '^src/modules/features/$1/',
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
