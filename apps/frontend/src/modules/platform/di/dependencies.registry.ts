import type { DomainRegistry } from './dependencies.context.ts';

/**
 * The registry, reachable without React.
 *
 * `DependenciesProvider` is still the door for React — a test mounting a subtree
 * with stub repositories relies on the context for isolation, and two renders in
 * one file must not see each other's registry. This module-level copy exists for
 * the consumers that have no context to read: a Svelte island mounted inside a
 * React page sees none of the React tree above it.
 *
 * Set once by the composition root, and by `DependenciesProvider` on mount so
 * the two can never disagree about what is wired.
 */
let registry: DomainRegistry | null = null;

export const setDependencyRegistry = (next: DomainRegistry | null): void => {
  registry = next;
};

export const getDependencyRegistry = (): DomainRegistry | null => registry;

/**
 * One module's domain, for code that runs outside React.
 *
 * Inside a component or a hook, call the feature's own accessor
 * (`useJobsDomain()`) instead: it reads the context, which is what makes a test
 * able to swap the registry for one subtree.
 */
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
