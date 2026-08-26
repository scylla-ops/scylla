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
  { id: Permission.READ_ORGANIZATION, label: msg`View the organization`, scope: ORGANIZATION },
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
    id: Permission.LIST_PROJECTS_BY_ORGANIZATION,
    label: msg`See every project in the organization`,
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
  { id: Permission.READ_PROJECT, label: msg`Open a project`, scope: PROJECT },
  {
    id: Permission.UPDATE_PROJECT,
    label: msg`Edit the project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    id: Permission.DELETE_PROJECT,
    label: msg`Delete the project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },

  // Pipelines
  {
    id: Permission.LIST_PIPELINES_BY_PROJECT,
    label: msg`View the pipeline list`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    id: Permission.READ_PIPELINE,
    label: msg`Open a pipeline`,
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
