// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { modules } from './registry.ts';

/**
 * Two conformance rules over the features themselves, enumerated from the
 * registry like the route rules in `module-permissions.test.ts`.
 *
 * Where that file walks *declarations*, this one reads *source*. The gating a
 * feature applies to its own buttons is not declared anywhere the type system
 * can see it, so the only way to ask "does this feature gate at all?" is to look
 * at what it wrote. That makes these rules deliberately coarse — they answer
 * completeness, never correctness:
 *
 *   caught     a feature that ships with no gating whatsoever, and a hook that
 *              crosses a feature boundary without checking for itself
 *   not caught the *wrong* permission on the right button
 *
 * Relating each mutation to the permission it needs would require that mapping
 * to be declared — today it is spread across a hook, a table, a child component
 * and a route. Until it is, the per-component tests carry correctness.
 */

const FEATURES_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'features');

const walk = (dir: string): string[] =>
  readdirSync(dir).flatMap(entry => {
    const path = join(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });

/** Source files only: a test file's mention of a permission proves nothing. */
const sourcesIn = (dir: string): string[] =>
  existsSync(dir) ? walk(dir).filter(f => /\.tsx?$/.test(f) && !f.includes('.test.')) : [];

const read = (path: string): string => readFileSync(path, 'utf8');

/** Module ids are the directory names under `features/`; this asserts it stays true. */
const featureDirs = modules.map(module => {
  const dir = join(FEATURES_DIR, module.id);
  return {
    id: module.id,
    dir,
    hooks: join(dir, 'presentation/hooks'),
    ui: join(dir, 'presentation/ui'),
  };
});

describe('feature permission conformance', () => {
  it('every registered module has a directory to scan', () => {
    const missing = featureDirs.filter(feature => !existsSync(feature.dir)).map(f => f.id);
    expect(
      missing,
      'a module id no longer matches its folder — these rules silently skip it',
    ).toEqual([]);
  });

  /**
   * Features that mutate without gating anything.
   *
   * A ratchet: entries may leave, never join without a reason that is a design
   * decision rather than a backlog item.
   */
  const UNGATED_FEATURES: Readonly<Record<string, string>> = {
    login:
      'Signing in is what establishes identity; there is no permission to hold before holding one.',
  };

  describe('a feature that mutates gates something in its UI', () => {
    const mutating = featureDirs.filter(feature =>
      sourcesIn(feature.hooks).some(file => read(file).includes('useMutation(')),
    );

    it('finds the mutating features it is supposed to check', () => {
      expect(mutating.length).toBeGreaterThan(5);
    });

    it.each(mutating.map(feature => [feature.id, feature] as const))('%s', (id, feature) => {
      const gates = sourcesIn(feature.ui).some(file =>
        /Permission\.[A-Z_]+|PermissionButton|useCan|useAuthorization/.test(read(file)),
      );

      if (id in UNGATED_FEATURES) {
        expect(
          gates,
          `feature "${id}" now gates part of its UI — drop it from UNGATED_FEATURES, the list only shrinks.`,
        ).toBe(false);
        return;
      }

      expect(
        gates,
        `feature "${id}" performs mutations but no component under its presentation/ui/ ever ` +
          'mentions a Permission. Every write it offers is available to anyone who can reach ' +
          'the page. Gate the actions, or add the feature to UNGATED_FEATURES with the reason.',
      ).toBe(true);
    });
  });

  /**
   * A hook another feature consumes runs outside its owner's route guard: the
   * consumer's page was entered on the consumer's permission, not the owner's.
   * `useJobsByPipelines` is the one that already learned this — it checks
   * `LIST_JOBS_BY_PIPELINE` itself rather than trusting whoever called it.
   */
  describe('a query hook consumed across a feature boundary checks for itself', () => {
    /**
     * What a read looks like on either side of the migration.
     *
     * A React feature exports a `use*` hook; a migrated one exports a
     * `*Queries` factory of options objects. The rule is the same for both —
     * the consumer's route guard does not cover the owner's permission — so
     * both shapes are collected here rather than the rule quietly emptying out
     * as features move to Svelte.
     */
    const isSharedRead = (name: string) => /^use[A-Z]/.test(name) || /Queries$/.test(name);

    /** Read names a feature re-exports through its barrel, by owning feature. */
    const barrelExports = new Map(
      featureDirs.map(feature => {
        const barrel = join(feature.dir, 'index.ts');
        const names = existsSync(barrel)
          ? [...read(barrel).matchAll(/\b(?:use[A-Z]\w*|\w+Queries)\b/g)].map(match => match[0])
          : [];
        return [feature.id, new Set(names)];
      }),
    );

    /** `owner.hookName` -> the features that import it, excluding the owner. */
    const consumersOf = new Map<string, Set<string>>();
    for (const feature of featureDirs) {
      for (const file of sourcesIn(feature.dir)) {
        const imports = read(file).matchAll(
          /import\s*(?:type\s*)?\{([^}]*)\}\s*from\s*'@\/modules\/features\/([a-z-]+)'/g,
        );
        for (const [, names, owner] of imports) {
          if (owner === feature.id) continue;
          for (const raw of names.split(',')) {
            const name = raw.trim().replace(/^type\s+/, '');
            if (!isSharedRead(name)) continue;
            const key = `${owner}.${name}`;
            consumersOf.set(key, (consumersOf.get(key) ?? new Set()).add(feature.id));
          }
        }
      }
    }

    interface SharedHook {
      readonly key: string;
      readonly consumers: readonly string[];
      readonly selfGated: boolean;
    }

    /** Collects the exported reads of one file that cross a barrel. */
    const sharedReadsIn = (
      featureId: string,
      file: string,
      exportPattern: RegExp,
      gatePattern: RegExp,
    ): SharedHook[] => {
      const source = read(file);
      const exported = barrelExports.get(featureId) ?? new Set<string>();

      return [...source.matchAll(exportPattern)]
        .map(match => match[1])
        .filter(name => exported.has(name))
        .flatMap(name => {
          const consumers = consumersOf.get(`${featureId}.${name}`);
          if (!consumers) return [];
          return [
            {
              key: `${featureId}.${name}`,
              consumers: [...consumers],
              selfGated: gatePattern.test(source),
            },
          ];
        });
    };

    const sharedHooks: SharedHook[] = featureDirs.flatMap(feature => [
      // The React half: a `use*` hook under `presentation/hooks/`.
      ...sourcesIn(feature.hooks).flatMap(file =>
        // `useQueryClient` is not a query — match the call, not the prefix.
        /\buseQuery\(|\buseQueries\(/.test(read(file))
          ? sharedReadsIn(
              feature.id,
              file,
              /export const (use[A-Z]\w*)/g,
              /useAuthorization|useCan/,
            )
          : [],
      ),
      // The Svelte half: a `*.queries.ts` factory. There is no hook to look at,
      // so the gate is a `can(...)` inside the options it builds.
      ...sourcesIn(join(feature.dir, 'presentation'))
        .filter(file => file.endsWith('.queries.ts'))
        .flatMap(file =>
          sharedReadsIn(feature.id, file, /export const (\w+Queries)\b/g, /\bcan\(/),
        ),
    ]);

    /**
     * Cross-feature hooks that deliberately do not check, and why. A ratchet.
     *
     * `TRIAGE` marks the ones nobody has ruled on yet: the consumer's route
     * permission is not the one the hook's data needs, so the call may well come
     * back `PERMISSION_DENIED`. They are listed rather than fixed because each
     * needs a product decision — what should the consuming page show instead?
     */
    const UNCHECKED_SHARED_HOOKS: Readonly<Record<string, string>> = {
      'jobs.useOrganizationJobs':
        'Organization-scoped and filtered server-side. The per-project fan-out it replaced cost ' +
        'one PERMISSION_DENIED toast per unreadable project — see use-org-overview.ts.',
      'pipeline.useOrganizationPipelines':
        'Same org-scoped, server-filtered read as useOrganizationJobs.',
      'organization.organizationQueries':
        'One entry where there were three hooks, and the same reasons. `members` is consumed by ' +
        'membership, whose route requires LIST_ORGANIZATION_MEMBERS — the permission the read ' +
        'needs, so the route guard covers it. TRIAGE for `mine`: roles consumes it on a route ' +
        'requiring MANAGE_ROLES, and listing organizations is a different permission.',
      'secret.secretQueries':
        'TRIAGE: consumed by pipeline\'s step dialog to offer secret names, on an editor route ' +
        'requiring UPDATE_PIPELINE rather than LIST_SECRETS. Newly listed rather than newly ' +
        'true — `useSecrets` crossed the same boundary before the factory replaced it.',
      'roles.useGrantableRoles':
        'TRIAGE: consumed by membership, whose route requires LIST_ORGANIZATION_MEMBERS rather ' +
        'than the grant permissions this read needs.',
    };

    it('finds the shared hooks it is supposed to check', () => {
      expect(sharedHooks.length).toBeGreaterThan(5);
    });

    it.each(sharedHooks.map(hook => [hook.key, hook] as const))('%s', (key, hook) => {
      if (key in UNCHECKED_SHARED_HOOKS) {
        expect(
          hook.selfGated,
          `${key} now checks for itself — drop it from UNCHECKED_SHARED_HOOKS, the list only shrinks.`,
        ).toBe(false);
        return;
      }

      expect(
        hook.selfGated,
        `${key} is consumed by ${hook.consumers.join(', ')}, so it runs outside its own route ` +
          'guard: those pages were entered on their permission, not this one. Gate the query ' +
          'with `enabled: ready && can(...)` as useJobsByPipelines does, or list it in ' +
          'UNCHECKED_SHARED_HOOKS with the reason.',
      ).toBe(true);
    });

    it('the unchecked list names only hooks that still cross a boundary', () => {
      const live = new Set(sharedHooks.map(hook => hook.key));
      const stale = Object.keys(UNCHECKED_SHARED_HOOKS).filter(key => !live.has(key));

      expect(
        stale,
        'these UNCHECKED_SHARED_HOOKS entries are no longer shared — delete them',
      ).toEqual([]);
    });
  });
});
