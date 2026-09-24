import { createStore } from '@shared/presentation/stores/create-store.ts';
import type { EffectivePermissionsEntity } from '@platform/authz/domain/entities/effective-permissions.entity.ts';

interface PermissionsState {
  /**
   * Effective permissions of the signed-in user. `null` until the first load.
   * The gates read this value synchronously, so no gate calls the backend.
   */
  permissions: EffectivePermissionsEntity | null;
  setPermissions: (permissions: EffectivePermissionsEntity | null) => void;
}

/**
 * The effective permissions of the signed-in user.
 *
 * Not persisted: each new session starts with unknown permissions (denied) and
 * loads them again. `syncMyPermissions` in `features/roles` is the only writer.
 */
export const permissionsStore = createStore<PermissionsState>(set => ({
  permissions: null,
  setPermissions: permissions => set({ permissions }),
}));
