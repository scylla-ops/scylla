import { useContext } from 'react';
import { DependenciesContext } from './dependencies.context.ts';

/**
 * Reads one module's use cases out of the injected registry.
 *
 * Call it through the feature's own accessor (`useJobsDomain()`), never
 * directly: that accessor is what supplies `TDomain`, so the cast here is
 * pinned to the module's real shape at exactly one place per feature.
 */
export const useModuleDomain = <TDomain extends object>(moduleId: string): TDomain => {
  const registry = useContext(DependenciesContext);

  if (registry == null) {
    throw new Error('useModuleDomain must be used within a DependenciesProvider');
  }

  const domain = registry[moduleId];

  if (domain == null) {
    throw new Error(`No module registered under id "${moduleId}"`);
  }

  return domain as TDomain;
};
