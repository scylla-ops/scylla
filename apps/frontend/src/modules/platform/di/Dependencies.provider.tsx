import type { ReactNode } from 'react';
import { DependenciesContext, type DomainRegistry } from './dependencies.context.ts';

interface DependenciesProviderProps {
  /** Assembled by the composition root — see `core/di/dependencies.ts`. */
  registry: DomainRegistry;
  children: ReactNode;
}

/**
 * Injects the module registry. Taking it as a prop rather than importing it is
 * what lets this provider live below the features: it also makes a test able to
 * mount a subtree with stub use cases.
 */
export const DependenciesProvider = ({ registry, children }: DependenciesProviderProps) => (
  <DependenciesContext.Provider value={registry}>{children}</DependenciesContext.Provider>
);
