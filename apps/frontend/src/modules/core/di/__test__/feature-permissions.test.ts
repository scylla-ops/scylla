// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { modules } from '../registry.ts';

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

const FEATURES_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', 'features');

const walk = (dir: string): string[] =>
  readdirSync(dir).flatMap(entry => {
    const path = join(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });

/**
 * Source files only: a test file's mention of a permission proves nothing.
 *
 * `.svelte` is in the glob, and has to be: a migrated feature gates its buttons
 * in components, so a `.tsx?`-only scan would quietly report every one of them
 * as ungated — or, worse, find nothing to check at all. See `refacto_svelte.md`
 * §4.6: no gate may go blind on the new code.
 */
const sourcesIn = (dir: string): string[] =>
  existsSync(dir) ? walk(dir).filter(f => /\.(tsx?|svelte)$/.test(f) && !/\.(test|fixture)\./.test(f)) : [];

const read = (path: string): string => readFileSync(path, 'utf8');

/**
 * The same file with its comments stripped.
 *
 * Every probe below is a bare identifier — `can(`, `useCan`, `useMutation(` —
 * and a doc comment explaining why a query needs no gate contains them exactly
 * as readily as a call does. `roles.queries.ts` notes that "every `can()` in the
 * app reads it" and was counted as self-gating on the strength of that sentence.
 * Prose is not a gate, and the failure mode is the silent one: a rule that finds
 * what it was looking for in a comment never fails.
 */
const code = (path: string): string =>
  read(path)
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1');

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
    /**
     * What a write looks like on either side of the migration.
     *
     * A React feature calls `useMutation(` in a hook under `presentation/hooks/`;
     * a migrated one declares `mutationOptions(` in its `*.queries.ts`. Both
     * shapes are collected, or the rule would empty itself out one feature at a
     * time as they move to Svelte — silently, since nothing fails when a
     * conformance test simply stops finding anything to check.
     */
    const writesIn = (feature: (typeof featureDirs)[number]) =>
      sourcesIn(feature.hooks).some(file => code(file).includes('useMutation(')) ||
      sourcesIn(join(feature.dir, 'presentation'))
        .filter(file => file.endsWith('.queries.ts') || file.endsWith('.mutations.ts'))
        .some(file => code(file).includes('mutationOptions('));

    const mutating = featureDirs.filter(writesIn);

    it('finds the mutating features it is supposed to check', () => {
      expect(mutating.length).toBeGreaterThan(5);
    });

    it.each(mutating.map(feature => [feature.id, feature] as const))('%s', (id, feature) => {
      // `can(` covers the Svelte half, where there is no hook to name.
      const gates = sourcesIn(feature.ui).some(file =>
        /Permission\.[A-Z_]+|PermissionButton|useCan|useAuthorization|\bcan\(/.test(code(file)),
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
          ? [...code(barrel).matchAll(/\b(?:use[A-Z]\w*|\w+Queries)\b/g)].map(match => match[0])
          : [];
        return [feature.id, new Set(names)];
      }),
    );

    /** `owner.hookName` -> the features that import it, excluding the owner. */
    const consumersOf = new Map<string, Set<string>>();
    for (const feature of featureDirs) {
      for (const file of sourcesIn(feature.dir)) {
        const imports = code(file).matchAll(
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
      const source = code(file);
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
        /\buseQuery\(|\buseQueries\(/.test(code(file))
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
      'jobs.jobQueries':
        'The entry `useOrganizationJobs` left behind. `byOrganization`, the read that crosses ' +
        'the barrel, is organization-scoped and filtered server-side; the per-project fan-out it ' +
        'replaced cost one PERMISSION_DENIED toast per unreadable project — see ' +
        'use-org-overview.ts. The fan-out that *does* check lives in its own file precisely so ' +
        'its `can(` does not vouch for this one.',
      'pipeline.pipelineQueries':
        'The entry `useOrganizationPipelines` left behind, for the same reason as jobQueries: ' +
        '`byOrganization` is the one read that crosses the barrel, and ' +
        '`ListOrganizationPipelines` is scoped server-side, so there is nothing to gate ' +
        "client-side. The hook is now this factory's React binding and goes in Phase 5.",
      'organization.organizationQueries':
        'One entry where there were three hooks, and the same reasons. `members` is consumed by ' +
        'membership, whose route requires LIST_ORGANIZATION_MEMBERS — the permission the read ' +
        'needs, so the route guard covers it. TRIAGE for `mine`: roles consumes it on a route ' +
        'requiring MANAGE_ROLES, and listing organizations is a different permission.',
      'secret.secretQueries':
        "TRIAGE: consumed by pipeline's step dialog to offer secret names, on an editor route " +
        'requiring UPDATE_PIPELINE rather than LIST_SECRETS. Newly listed rather than newly ' +
        'true — `useSecrets` crossed the same boundary before the factory replaced it.',
      'roles.roleQueries':
        'The entry `useGrantableRoles` left behind, and a decision rather than debt: every read ' +
        "this factory exposes across the barrel takes the caller's gate as `enabled`, which is " +
        'the only place the answer is known. `grantable` needs no permission at all — the ' +
        'backend serves a compile-time constant — while `catalog` and `scopedGrants` are asked ' +
        'for only when the consumer already holds MANAGE_ROLES or MANAGE_*_GRANTS, which is ' +
        'what membership passes (see assignable-roles.state.svelte.ts). Gating inside the ' +
        'factory would mean naming one permission for three reads that need three.',
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
