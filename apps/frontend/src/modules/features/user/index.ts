/** `loadUserSettingsPage` is a loader: `organization` renders the page behind its own route. */
export type { UserEntity } from './domain/entities/user.entity.ts';
export {
  canListUsers,
  userMutations,
  userQueries,
  USERS_QUERY_KEY,
  USER_QUERY_KEY,
} from './presentation/user.queries.ts';
export const loadUserSettingsPage = () =>
  import('./presentation/ui/settings/UserSettings.page.svelte');
