/**
 * Machine identities (apps) and the secrets they authenticate with.
 *
 * The public API of the module. Nothing outside consumes it yet; the domain
 * types are exposed because they are the stable part and cost nothing at
 * runtime (types are erased), and the query factories because that is what a
 * consumer would reach for — `useApps` and its siblings are gone.
 */
export type { AppEntity, AppSecretEntity } from './domain/entities/app.entity.ts';
export type { CreatedApp, CreatedAppSecret } from './domain/structs/app.struct.ts';
export {
  appQueries,
  appMutations,
  APPS_QUERY_KEY,
  APP_QUERY_KEY,
  APP_SECRETS_QUERY_KEY,
} from './presentation/apps.queries.ts';
