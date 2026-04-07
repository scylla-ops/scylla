import { createContext } from 'react';
import type { Dependencies } from '@core/di/Dependencies.ts';

//TODO: use a lib

export const DependenciesContext = createContext<Dependencies | null>(null);
