/** Project-scoped secrets, injected into pipeline runs. */
export type { SecretEntity, CreateSecretInput } from './domain/entities/secret.entity.ts';
export {
  secretQueries,
  secretMutations,
  SECRETS_QUERY_KEY,
  type CreateSecretValues,
} from './presentation/secret.queries.ts';
