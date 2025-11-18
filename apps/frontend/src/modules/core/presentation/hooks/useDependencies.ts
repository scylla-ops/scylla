import { useContext } from 'react';
import { DependenciesContext } from '@core/presentation/contexts/DependenciesContext.ts';

export const useDependencies = () => {
  const context = useContext(DependenciesContext);

  if (context == null) {
    throw new Error('useDependencies must be used within a DependenciesProvider');
  }

  return context;
};
