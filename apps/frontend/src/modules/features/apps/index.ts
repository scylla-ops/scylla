/**
 * Machine identities (apps) and the secrets they authenticate with.
 *
 * The public API of the module. Nothing outside consumes it yet; the domain
 * types are exposed because they are the stable part and cost nothing at
 * runtime (types are erased).
 */
export type { AppEntity, AppSecretEntity } from './domain/entities/app.entity.ts';
export type { CreatedApp, CreatedAppSecret } from './domain/structs/app.struct.ts';
export { useApps, useApp, useAppSecrets } from './presentation/hooks/use-apps.ts';
