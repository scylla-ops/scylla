import { useModuleDomain } from '@platform/di/index.ts';
import type { UserModule } from '../../user.module.ts';

/** Typed access to the user module's use cases. */
export const useUserDomain = () => useModuleDomain<typeof UserModule.domain>('user');
