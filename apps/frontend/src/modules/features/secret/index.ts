/** Project-scoped secrets, injected into pipeline runs. */
export type { SecretEntity, CreateSecretInput } from './domain/entities/secret.entity.ts';
export { useSecrets, useCreateSecret, useDeleteSecret } from './presentation/hooks/use-secrets.ts';
