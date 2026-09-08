/**
 * User accounts: the directory and a user's own settings.
 *
 * The public API of the module. `UserSettingsPage` is exported because
 * `organization` composes it behind its own route to fill the organizations
 * panel — a slot, not a leak, and its consumer is lazily loaded.
 */
export type { UserEntity } from './domain/entities/user.entity.ts';
export { useUser } from './presentation/hooks/use-user.ts';
export { useUsers } from './presentation/hooks/use-users.ts';
export { UserSettingsPage } from './presentation/ui/settings/UserSettings.page.tsx';
