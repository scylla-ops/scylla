import { createContext } from 'react';

/**
 * Module id -> that module's `domain` (its use-case instances).
 *
 * Deliberately untyped per module: the concrete map is assembled by the
 * composition root, and if this context named it, every feature reading a
 * dependency would transitively depend on every other feature. Features pin the
 * type on their own side through `useModuleDomain<T>` — see each feature's
 * `di/use-*-domain.ts`.
 */
export type DomainRegistry = Readonly<Record<string, object>>;

export const DependenciesContext = createContext<DomainRegistry | null>(null);
