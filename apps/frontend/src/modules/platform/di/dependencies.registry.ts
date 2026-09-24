/**
 * Module id -> that module's `domain` (its repositories and use cases).
 *
 * Deliberately untyped per module: the concrete map is assembled by the
 * composition root, and if this type named it, every feature reading a
 * dependency would depend on every other feature. A feature pins the type on
 * its own side when it calls `getModuleDomain<T>`.
 */
export type DomainRegistry = Readonly<Record<string, object>>;

/**
 * The registry of the app. The composition root sets it at start-up. A test sets
 * it with `withRegistry` from `src/test/render.svelte.ts`.
 */
let registry: DomainRegistry | null = null;

export const setDependencyRegistry = (next: DomainRegistry | null): void => {
  registry = next;
};

/** The domain of one module. Call it from a `*.queries.ts` or a `*.state.svelte.ts`, not from a component. */
export const getModuleDomain = <TDomain extends object>(moduleId: string): TDomain => {
  if (registry == null) {
    throw new Error(
      'No dependency registry set. The composition root installs it at start-up; ' +
        'a test must call `setDependencyRegistry` before reaching the domain.',
    );
  }

  const domain = registry[moduleId];

  if (domain == null) {
    throw new Error(`No module registered under id "${moduleId}"`);
  }

  return domain as TDomain;
};
