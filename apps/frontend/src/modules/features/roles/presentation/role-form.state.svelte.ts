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

/**
 * Creating or editing one role.
 *
 * Seeded from `role` **at construction**, with no effect watching it: the
 * dialog renders this form under `{#key open}`, so reopening it builds a new
 * instance and the initialisers below are the reset. React needed an effect on
 * `[open, role]` for the same thing, plus a `setTimeout` to undo the mutation's
 * success flag before the next opening.
 */
export const createRoleForm = (role: RoleEntity | null) => {
  const isEdit = role !== null;
  const initialScope = role?.scope ?? PermissionScope.ORGANIZATION;

  let name = $state(role?.name ?? '');
  let description = $state(role?.description ?? '');
  let scope = $state<PermissionScope>(initialScope);
  let accessKind = $state<AccessKind>(
    role?.access.kind === 'fullControl' ? 'fullControl' : 'restricted',
  );

  /**
   * The ticked boxes. Hidden and non-catalog permissions are kept out: they are
   * re-added on save, and leaving them in would show them twice — once in the
   * tree, once in the count.
   */
  let permissions = $state<Permission[]>(
    role?.access.kind === 'restricted'
      ? role.access.permissions.filter(
          permission =>
            isEditablePermission(permission) && !isHiddenAtScope(permission, initialScope),
        )
      : [],
  );

  /**
   * Permissions the role holds that this build's catalog does not expose — set
   * by the backend, by another client, or by an older build. The editor cannot
   * show them, so it carries them through rather than quietly deleting them.
   */
  const preserved: Permission[] =
    role?.access.kind === 'restricted'
      ? role.access.permissions.filter(permission => !isEditablePermission(permission))
      : [];

  const createRole = createMutation(() => roleMutations.create());
  const updateRole = createMutation(() => roleMutations.update());

  /** What the role will actually confer: the ticked boxes plus the implicit ones. */
  const conferred = $derived(withImplicitPermissions(scope, permissions));

  const buildAccess = (): AccessSpec =>
    accessKind === 'fullControl'
      ? { kind: 'fullControl' }
      : {
          // The implicit ones never appear in the editor, so they are written
          // here or nowhere.
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
    /**
     * Changing scope drops every selected permission the new scope cannot
     * confer, and every one it now confers implicitly — those are re-added on
     * save and would otherwise be counted twice.
     */
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
    /**
     * Writes the role and answers whether it landed, so the dialog closes on
     * success and stays open — with the typing still in it — on failure. The
     * error itself is already toasted by the global mutation cache.
     */
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
