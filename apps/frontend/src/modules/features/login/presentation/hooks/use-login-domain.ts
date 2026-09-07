import { useModuleDomain } from '@platform/di';
import type { LoginModule } from '../../login.module.ts';

/** Typed access to the login module's use cases. */
export const useLoginDomain = () => useModuleDomain<typeof LoginModule.domain>('login');
