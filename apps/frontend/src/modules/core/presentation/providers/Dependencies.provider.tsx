import * as React from 'react';
import { DependenciesContext } from '@core/presentation/contexts/dependencies.context.ts';

import { dependencies } from '@core/di/dependencies.ts';

export const DependenciesProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <DependenciesContext.Provider value={dependencies}>{children}</DependenciesContext.Provider>
  );
};
