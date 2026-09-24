import { PermissionScope, type AccessSpec, type Permission } from '@platform/authz';
import { createMutation } from '@platform/query';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { roleMutations } from './roles.queries.ts';
import {
  getPermissionsForScope,
  isEditablePermission,
  isHiddenAtScope,
  withImplicitPermissions,
} from './utils/permission-mapping.ts';

export type AccessKind = 'fullControl' | 'restricted';

/** Creates or edits a role. Seeded once: the dialog rebuilds it at each opening. */
export const createRoleForm = (role: RoleEntity | null) => {
  const isEdit = role !== null;
  const initialScope = role?.scope ?? PermissionScope.ORGANIZATION;

  let name = $state(role?.name ?? '');
  let description = $state(role?.description ?? '');
  let scope = $state<PermissionScope>(initialScope);
  let accessKind = $state<AccessKind>(
    role?.access.kind === 'fullControl' ? 'fullControl' : 'restricted',
  );

  /** The ticked boxes, without the hidden and preserved permissions (re-added on save). */
  let permissions = $state<Permission[]>(
    role?.access.kind === 'restricted'
      ? role.access.permissions.filter(
          permission =>
            isEditablePermission(permission) && !isHiddenAtScope(permission, initialScope),
        )
      : [],
  );

  /** Permissions this build's catalog does not show: kept on save, never deleted. */
  const preserved: Permission[] =
    role?.access.kind === 'restricted'
      ? role.access.permissions.filter(permission => !isEditablePermission(permission))
      : [];

  const createRole = createMutation(() => roleMutations.create());
  const updateRole = createMutation(() => roleMutations.update());

  /** The ticked boxes plus the implicit ones. */
  const conferred = $derived(withImplicitPermissions(scope, permissions));

  const buildAccess = (): AccessSpec =>
    accessKind === 'fullControl'
      ? { kind: 'fullControl' }
      : {
          // The implicit permissions never show in the editor: add them here.
          kind: 'restricted',
          // deduplication,
          // spread straight back into an array; the Set never outlives the expression.
          // eslint-disable-next-line svelte/prefer-svelte-reactivity
          permissions: [...new Set([...preserved, ...conferred])],
        };

  return {
    get isEdit() {
      return isEdit;
    },
    get name() {
      return name;
    },
    set name(next: string) {
      name = next;
    },
    get description() {
      return description;
    },
    set description(next: string) {
      description = next;
    },
    get scope() {
      return scope;
    },
    get accessKind() {
      return accessKind;
    },
    set accessKind(next: AccessKind) {
      accessKind = next;
    },
    get permissions() {
      return permissions;
    },
    set permissions(next: Permission[]) {
      permissions = next;
    },
    get preservedCount() {
      return preserved.length;
    },
    get conferredCount() {
      return conferred.length;
    },
    get isPending() {
      return createRole.isPending || updateRole.isPending;
    },
    /** An organization role is never empty: it always carries what belonging means. */
    get isValid() {
      return (
        name.trim().length > 0 &&
        (accessKind === 'fullControl' || conferred.length + preserved.length > 0)
      );
    },
    /** Drops the permissions the new scope cannot confer, and the ones it now confers implicitly. */
    changeScope: (next: PermissionScope) => {
      scope = next;
      // a lookup local to
      // this handler, gone by the time it returns.
      // eslint-disable-next-line svelte/prefer-svelte-reactivity
      const allowed = new Set(getPermissionsForScope(next) ?? []);
      permissions = permissions.filter(
        permission => allowed.has(permission) && !isHiddenAtScope(permission, next),
      );
    },
    /** Returns whether it was saved: the dialog closes on success and keeps the input on failure. */
    submit: async (): Promise<boolean> => {
      try {
        if (role) {
          await updateRole.mutateAsync({
            id: role.id,
            name: name.trim(),
            description: description.trim(),
            access: buildAccess(),
          });
        } else {
          await createRole.mutateAsync({
            name: name.trim(),
            description: description.trim(),
            scope,
            access: buildAccess(),
          });
        }
        return true;
      } catch {
        return false;
      }
    },
  };
};

export type RoleForm = ReturnType<typeof createRoleForm>;
