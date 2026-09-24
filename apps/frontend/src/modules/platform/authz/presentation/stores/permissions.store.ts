import { createStore } from '@shared/presentation/stores/create-store.ts';
import type { EffectivePermissionsEntity } from '@platform/authz/domain/entities/effective-permissions.entity.ts';

interface PermissionsState {
  /** `null` until the first load. */
  permissions: EffectivePermissionsEntity | null;
  setPermissions: (permissions: EffectivePermissionsEntity | null) => void;
}

/** Not persisted. Written only by `syncMyPermissions` in `features/roles`. */
export const permissionsStore = createStore<PermissionsState>(set => ({
  permissions: null,
  setPermissions: permissions => set({ permissions }),
}));
