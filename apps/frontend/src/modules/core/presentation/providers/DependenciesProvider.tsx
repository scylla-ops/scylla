import { useContext } from 'react';
import * as React from 'react';
import { DependenciesContext } from '@/modules/core/presentation/contexts/DependenciesContext.ts';

export const DependenciesProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const dependencies = useContext(DependenciesContext);
  return (
    <DependenciesContext.Provider value={dependencies}>{children}</DependenciesContext.Provider>
  );
};
