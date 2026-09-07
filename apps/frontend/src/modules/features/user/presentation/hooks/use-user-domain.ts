import { useModuleDomain } from '@platform/di';
import type { UserModule } from '../../user.module.ts';

/** Typed access to the user module's use cases. */
export const useUserDomain = () => useModuleDomain<typeof UserModule.domain>('user');
