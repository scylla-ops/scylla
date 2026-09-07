import { useModuleDomain } from '@platform/di';
import type { SecretModule } from '../../secret.module.ts';

/** Typed access to the secret module's use cases. */
export const useSecretDomain = () => useModuleDomain<typeof SecretModule.domain>('secret');
