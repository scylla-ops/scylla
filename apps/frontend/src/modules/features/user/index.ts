/**
 * User accounts: the directory and a user's own settings.
 *
 * The public API of the module. `UserSettingsPage` is exported because
 * `organization` composes it behind its own route to fill the organizations
 * panel — a slot, not a leak, and its consumer is lazily loaded.
 *
 * `userQueries` replaced `useUser` / `useUsers`: the options objects are
 * framework-agnostic, so a module still on React passes them to react-query's
 * `useQuery` and shares this module's cache entries rather than forking them.
 */
export type { UserEntity } from './domain/entities/user.entity.ts';
export {
  canListUsers,
  userMutations,
  userQueries,
  USERS_QUERY_KEY,
  USER_QUERY_KEY,
} from './presentation/user.queries.ts';
export { default as UserSettingsPage } from './presentation/ui/settings/UserSettings.page.svelte';
