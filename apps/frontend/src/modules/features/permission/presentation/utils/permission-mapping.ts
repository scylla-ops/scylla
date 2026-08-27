import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';
import {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * The V1 permission catalog: the subset of the wire vocabulary a human may
 * toggle when building a role.
 *
 * It holds to one invariant, and the whole design depends on it:
 *
 *   **every permission the UI gates on is in this catalog.**
 *
 * Break it and you get pages no role built here can ever open — the failure
 * mode this catalog was rewritten to kill.
 *
 * The converse almost holds. Three entries are never checked client-side, and
 * all three are load-bearing on the server:
 *  - `READ_PROJECT` / `READ_PIPELINE` guard `GetProject` / `GetPipeline`, and
 *    `READ_PROJECT` is also what narrows the project list to the projects a
 *    caller holds a grant on;
 *  - `LIST_PROJECTS_BY_ORGANIZATION` widens that same list to every project of
 *    the organization. Reading the organization is what opens the page; this
 *    only decides how much of it you see.
 *
 * Everything outside the catalog (invitations, app/agent-side job writes, the
 * per-entity read permissions the backend checks on its own) still travels on
 * the wire and is preserved untouched when a role is edited — it is simply not
 * something this build asks a human to reason about.
 */
export interface PermissionDefinition {
  id: Permission;
  label: MessageDescriptor;
  /**
   * Used instead of `label` when the role conferring it is bound to a scope
   * broader than this permission's own. A project permission held by an
   * organization role does not apply to *a* project but to every project of the
   * organization, and the label has to say so — "Edit the project" in an
   * organization role reads as a narrowing that isn't there.
   *
   * Only needed where the singular is baked into the wording; entries already
   * phrased in the plural ("Run pipelines") read correctly at every scope.
   */
  broadLabel?: MessageDescriptor;
  /**
   * The narrowest scope at which the permission does something. A role bound to
   * that scope — or any broader one — may confer it. Mirrors the backend's
   * `AuthzAction.min_scope`.
   */
  scope: PermissionScope;
  /**
   * The permission that must be held for this one to mean anything, which is
   * also its parent in the role editor's tree. Single parent by design: it
   * keeps the tree well-formed and every node reachable exactly once.
   */
  dependsOn?: Permission;
}

const { SYSTEM, ORGANIZATION, PROJECT } = PermissionScope;

/**
 * Ordered by scope, then by the tree each root opens. The order is what the
 * role editor renders, so it reads top-down from broadest to narrowest.
 */
export const PERMISSION_CATALOG: PermissionDefinition[] = [
  // ── System ──────────────────────────────────────────────────────────────────
  { id: Permission.LIST_USERS, label: msg`View users`, scope: SYSTEM },
  {
    id: Permission.CREATE_USER,
    label: msg`Create users`,
    scope: SYSTEM,
    dependsOn: Permission.LIST_USERS,
  },
  {
    id: Permission.DELETE_USER,
    label: msg`Delete users`,
    scope: SYSTEM,
    dependsOn: Permission.LIST_USERS,
  },
  { id: Permission.CREATE_ORGANIZATION, label: msg`Create organizations`, scope: SYSTEM },
  { id: Permission.MANAGE_ROLES, label: msg`Manage roles`, scope: SYSTEM },
  {
    // The single grant-management permission in this build: roles are created
    // and handed out by system administrators, at every scope. The narrower
    // `MANAGE_ORG_GRANTS` / `MANAGE_PROJECT_GRANTS` exist on the wire but are
    // deliberately not offered here — delegating administration is post-V1.
    id: Permission.MANAGE_SYSTEM_GRANTS,
    label: msg`Grant and revoke roles`,
    scope: SYSTEM,
  },

  // ── Organization ────────────────────────────────────────────────────────────
  {
    // Not "a permission you may hand out" at organization scope — it is what
    // belonging *is*. See IMPLICIT_PERMISSIONS_BY_SCOPE. The label says so, and
    // is what the role detail panel shows for it.
    id: Permission.READ_ORGANIZATION,
    label: msg`Member of the organization`,
    // At system scope it is not membership at all — it reaches every tenant.
    broadLabel: msg`See every organization on the instance`,
    scope: ORGANIZATION,
  },
  {
    id: Permission.UPDATE_ORGANIZATION,
    label: msg`Edit the organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    id: Permission.DELETE_ORGANIZATION,
    label: msg`Delete the organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    id: Permission.LIST_AGENTS,
    label: msg`View agents`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    // Without it the project list is narrowed to the projects the holder has a
    // grant on — reading the organization is what opens the page at all.
    //
    // At organization scope it also stands in for READ_PROJECT, which is what
    // turns "sees them listed" into "can open them": see
    // IMPLICIT_PERMISSIONS_BY_SCOPE. The label covers both.
    id: Permission.LIST_PROJECTS_BY_ORGANIZATION,
    label: msg`See and open every project in the organization`,
    // The stand-in only holds at organization scope; a system role must still
    // ask for READ_PROJECT separately, so the wording drops "and open".
    broadLabel: msg`See every project of every organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    id: Permission.CREATE_PROJECT,
    label: msg`Create projects`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_PROJECTS_BY_ORGANIZATION,
  },

  // ── Project ─────────────────────────────────────────────────────────────────
  {
    // Visibility, not navigation: it is what puts a project in the holder's
    // list when they cannot list the whole organization. In an organization
    // role that already confers `LIST_PROJECTS_BY_ORGANIZATION` it adds
    // nothing; in a project role it is the entire point.
    id: Permission.READ_PROJECT,
    label: msg`View the project`,
    // Hidden in the organization editor (LIST_PROJECTS_BY_ORGANIZATION stands in
    // for it), so this shows in a system role's tree and in role detail panels —
    // it has to read correctly at both scopes.
    broadLabel: msg`Open any project`,
    scope: PROJECT,
  },
  {
    id: Permission.UPDATE_PROJECT,
    label: msg`Edit the project`,
    broadLabel: msg`Edit every project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    id: Permission.DELETE_PROJECT,
    label: msg`Delete the project`,
    broadLabel: msg`Delete every project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },

  // Pipelines
  {
    id: Permission.LIST_PIPELINES_BY_PROJECT,
    label: msg`View the pipeline list`,
    broadLabel: msg`View the pipelines of every project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    id: Permission.READ_PIPELINE,
    label: msg`Open a pipeline`,
    broadLabel: msg`Open any pipeline`,
    scope: PROJECT,
    dependsOn: Permission.LIST_PIPELINES_BY_PROJECT,
  },
  {
    id: Permission.CREATE_PIPELINE,
    label: msg`Create pipelines`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },
  {
    id: Permission.UPDATE_PIPELINE,
    label: msg`Edit pipelines`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },
  {
    id: Permission.DELETE_PIPELINE,
    label: msg`Delete pipelines`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },
  {
    id: Permission.RUN_PIPELINE,
    label: msg`Run pipelines`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },
  {
    id: Permission.MANAGE_TRIGGERS,
    label: msg`Manage triggers`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },

  // Jobs
  {
    id: Permission.LIST_JOBS_BY_PIPELINE,
    label: msg`View pipeline jobs`,
    scope: PROJECT,
    dependsOn: Permission.READ_PIPELINE,
  },
  {
    id: Permission.READ_JOB_LOGS,
    label: msg`Read job logs`,
    scope: PROJECT,
    dependsOn: Permission.LIST_JOBS_BY_PIPELINE,
  },
  {
    id: Permission.DELETE_JOB,
    label: msg`Delete jobs`,
    scope: PROJECT,
    dependsOn: Permission.LIST_JOBS_BY_PIPELINE,
  },

  // Secrets
  {
    id: Permission.LIST_SECRETS,
    label: msg`View project secrets`,
    broadLabel: msg`View the secrets of every project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    id: Permission.CREATE_SECRET,
    label: msg`Create secrets`,
    scope: PROJECT,
    dependsOn: Permission.LIST_SECRETS,
  },
  {
    id: Permission.DELETE_SECRET,
    label: msg`Delete secrets`,
    scope: PROJECT,
    dependsOn: Permission.LIST_SECRETS,
  },
];

/** Catalog entries by permission — `undefined` for anything outside the catalog. */
const DEFINITION_BY_PERMISSION = new Map<Permission, PermissionDefinition>(
  PERMISSION_CATALOG.map(definition => [definition.id, definition]),
);

export const getPermissionDefinition = (permission: Permission): PermissionDefinition | undefined =>
  DEFINITION_BY_PERMISSION.get(permission);

/** Whether a human may toggle this permission in the role editor. */
export const isEditablePermission = (permission: Permission): boolean =>
  DEFINITION_BY_PERMISSION.has(permission);

export const ALL_SCOPES: PermissionScope[] = [SYSTEM, ORGANIZATION, PROJECT];

export const SCOPE_LABELS: Record<PermissionScope, MessageDescriptor> = {
  [PermissionScope.SYSTEM]: msg`System`,
  [PermissionScope.ORGANIZATION]: msg`Organization`,
  [PermissionScope.PROJECT]: msg`Project`,
  [PermissionScope.UNSPECIFIED]: msg`Unknown`,
};

/**
 * The scopes whose permissions a role bound to `scope` may confer. A permission
 * is coherent at its own scope and at every broader (ancestor) one, so a
 * SYSTEM role can confer anything while a PROJECT role stays project-local.
 */
const SCOPE_HIERARCHY: Record<PermissionScope, PermissionScope[]> = {
  [PermissionScope.SYSTEM]: [SYSTEM, ORGANIZATION, PROJECT],
  [PermissionScope.ORGANIZATION]: [ORGANIZATION, PROJECT],
  [PermissionScope.PROJECT]: [PROJECT],
  [PermissionScope.UNSPECIFIED]: [],
};

/** The catalog entries a role bound to `scope` may confer, in catalog order. */
export const getPermissionDefinitionsForScope = (
  scope: PermissionScope,
): PermissionDefinition[] => {
  const allowed = SCOPE_HIERARCHY[scope] ?? [];
  return PERMISSION_CATALOG.filter(definition => allowed.includes(definition.scope));
};

/**
 * A permission the role editor never shows at a given scope, and writes on the
 * caller's behalf. Two shapes, told apart by `standsIn`.
 */
interface ImplicitPermission {
  /** Never a node, never a toggle, at the scope it is listed under. */
  id: Permission;
  /**
   * The node that takes its place: it adopts `id`'s children in the tree, and
   * ticking it writes `id` too. Absent → `id` is conferred unconditionally and
   * shown as a ticked, locked row above the tree.
   */
  standsIn?: Permission;
}

/**
 * What each scope confers without asking. Both entries exist because at
 * organization scope the permission in question is not a choice — it is what the
 * thing above it already means — and offering it as a checkbox only lets an
 * administrator build a role that is dead on arrival.
 *
 * `READ_ORGANIZATION` is unconditional. There is no membership table on the
 * backend: belonging to an organization *is* holding a grant at its scope, and
 * `readOrganization` is the entire content of the `organization-member` builtin
 * that admits people (`policies.cedar`: the old `org-member` policy is gone,
 * "what belonging used to confer is the `organization-member` builtin role").
 * Untickable, it would let someone be admitted to an organization they cannot
 * see.
 *
 * `READ_PROJECT` rides on `LIST_PROJECTS_BY_ORGANIZATION`, because the backend
 * splits the two across RPCs that only make sense together at this scope
 * (`project/use_case.rs`): `list_by_organization` widens the list to every
 * project once `listProjectsByOrganization` holds, so `readProject` adds nothing
 * to *visibility* — but `get` checks `readProject` unconditionally, so without it
 * the holder lists every project and opens none. An organization-scoped grant
 * conferring `readProject` covers every project of the organization (Cedar
 * `resource in ?resource`), which is what makes the pairing exact rather than
 * approximate.
 *
 * Neither applies at SYSTEM scope, where both are genuinely separable: there
 * they mean "every organization / every project on the instance", capabilities
 * an administrator should have to ask for. PROJECT scope is untouched — a
 * project role is about one project, and `READ_PROJECT` is its whole point.
 */
const IMPLICIT_PERMISSIONS_BY_SCOPE: Partial<Record<PermissionScope, ImplicitPermission[]>> = {
  [PermissionScope.ORGANIZATION]: [
    { id: Permission.READ_ORGANIZATION },
    { id: Permission.READ_PROJECT, standsIn: Permission.LIST_PROJECTS_BY_ORGANIZATION },
    { id: Permission.READ_PIPELINE, standsIn: Permission.LIST_PIPELINES_BY_PROJECT },
  ],
  [PermissionScope.PROJECT]: [
    { id: Permission.READ_PIPELINE, standsIn: Permission.LIST_PIPELINES_BY_PROJECT },
  ],
};

const getImplicitAtScope = (scope: PermissionScope): ImplicitPermission[] =>
  IMPLICIT_PERMISSIONS_BY_SCOPE[scope] ?? [];

/**
 * Conferred unconditionally at `scope` — the ticked, locked rows the editor
 * shows above the tree.
 */
export const getAlwaysGrantedPermissionsForScope = (scope: PermissionScope): Permission[] =>
  getImplicitAtScope(scope)
    .filter(entry => entry.standsIn === undefined)
    .map(entry => entry.id);

/** Whether the editor hides `permission` at `scope`, for either reason. */
export const isHiddenAtScope = (permission: Permission, scope: PermissionScope): boolean =>
  getImplicitAtScope(scope).some(entry => entry.id === permission);

/**
 * Everything a role bound to `scope` confers once `selected` is ticked: the
 * visible choices, plus what the scope confers by construction, plus the riders
 * carried by a stand-in that was ticked. This is what gets written — the editor
 * never shows the difference, so nothing else may compute it.
 */
export const withImplicitPermissions = (
  scope: PermissionScope,
  selected: Permission[],
): Permission[] => {
  const conferred = new Set(selected);
  for (const entry of getImplicitAtScope(scope)) {
    if (entry.standsIn === undefined || conferred.has(entry.standsIn)) conferred.add(entry.id);
  }
  return [...conferred];
};

/**
 * {@link getPermissionDefinitionsForScope} minus the hidden entries — what the
 * role editor actually renders.
 *
 * A child of a hidden entry is re-hung from whatever stands in for it, so the
 * subtree keeps its shape instead of scattering into roots. With no stand-in the
 * children become roots, which `buildPermissionTree` already handles.
 */
export const getEditablePermissionDefinitionsForScope = (
  scope: PermissionScope,
): PermissionDefinition[] => {
  const implicit = getImplicitAtScope(scope);
  const hidden = new Set(implicit.map(entry => entry.id));
  const standInFor = new Map(
    implicit
      .filter(entry => entry.standsIn !== undefined)
      .map(entry => [entry.id, entry.standsIn!] as const),
  );

  return getPermissionDefinitionsForScope(scope)
    .filter(definition => !hidden.has(definition.id))
    .map(definition => {
      if (definition.dependsOn === undefined) return definition;
      const substitute = standInFor.get(definition.dependsOn);
      return substitute === undefined ? definition : { ...definition, dependsOn: substitute };
    });
};

/** Same as {@link getPermissionDefinitionsForScope}, reduced to the ids. */
export const getPermissionsForScope = (scope: PermissionScope): Permission[] =>
  getPermissionDefinitionsForScope(scope).map(definition => definition.id);

/**
 * A readable name for a permission the catalog doesn't cover, derived from the
 * enum key (`LIST_JOBS_BY_PIPELINE` → `List jobs by pipeline`). Untranslated,
 * but honest — far better than showing every uncatalogued permission as
 * "Unknown".
 */
export const humanizePermission = (permission: Permission): string => {
  const key = Permission[permission] as string | undefined;
  if (!key) return `Permission #${permission}`;
  const words = key.toLowerCase().replace(/_/g, ' ');
  return words.charAt(0).toUpperCase() + words.slice(1);
};
