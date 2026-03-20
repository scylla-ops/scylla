import * as React from 'react';
import { DependenciesContext } from '@core/presentation/contexts/DependenciesContext.ts';

import { dependencies } from '@core/di/Dependencies.ts';

export const DependenciesProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <DependenciesContext.Provider value={dependencies}>{children}</DependenciesContext.Provider>
  );
};
