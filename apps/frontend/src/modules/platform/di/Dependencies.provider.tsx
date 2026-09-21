import { useEffect, type ReactNode } from 'react';
import { DependenciesContext, type DomainRegistry } from './dependencies.context.ts';
import { getDependencyRegistry, setDependencyRegistry } from './dependencies.registry.ts';

interface DependenciesProviderProps {
  /** Assembled by the composition root — see `core/di/dependencies.ts`. */
  registry: DomainRegistry;
  children: ReactNode;
}

/**
 * Injects the module registry. Taking it as a prop rather than importing it is
 * what lets this provider live below the features: it also makes a test able to
 * mount a subtree with stub use cases.
 *
 * It installs the **module singleton** as well as the context, which is not
 * belt-and-braces. Since Phase 3 a React hook may be a thin binding over a
 * `*.queries.ts` factory — `useScopedGrants` is one — and a factory resolves its
 * repository through `getModuleDomain()`, which reads the singleton and knows
 * nothing about this tree. Without the line below, a test that injects a fake
 * repository here watches the hook reach for the real one and throw.
 *
 * The write happens during render rather than in the effect on purpose: a child
 * can call a factory on its very first render, which is before any effect runs.
 */
export const DependenciesProvider = ({ registry, children }: DependenciesProviderProps) => {
  const previous = getDependencyRegistry();
  if (previous !== registry) setDependencyRegistry(registry);

  // Module state outlives the tree, so it has to be taken back out — otherwise
  // the next test in the file inherits this one's stubs.
  useEffect(() => () => setDependencyRegistry(previous), [previous]);

  return <DependenciesContext.Provider value={registry}>{children}</DependenciesContext.Provider>;
};
