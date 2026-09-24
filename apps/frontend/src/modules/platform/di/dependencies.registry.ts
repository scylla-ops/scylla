/**
 * Module id -> that module's `domain`. Untyped on purpose: a feature types its
 * own domain with `getModuleDomain<T>`, so no feature depends on the others.
 */
export type DomainRegistry = Readonly<Record<string, object>>;

let registry: DomainRegistry | null = null;

export const setDependencyRegistry = (next: DomainRegistry | null): void => {
  registry = next;
};

/** Call it from a `*.queries.ts` or a `*.state.svelte.ts`, never from a component. */
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
