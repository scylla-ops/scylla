import { msg } from '@lingui/core/macro';
import { i18n, type MessageDescriptor } from '@lingui/core';
import { Permission, PermissionScope, } from '@platform/authz';

/**
 * The permissions a human may toggle when building a role. Invariant: every
 * permission the UI gates on is in this catalog. Permissions outside it are kept
 * untouched when a role is edited.
 */
export interface PermissionDefinition {
  id: Permission;
  label: MessageDescriptor;
  /** Used when the role's scope is broader than this permission's: "every project", not "the project". */
  broadLabel?: MessageDescriptor;
  /** The narrowest scope where it does something (backend `AuthzAction.min_scope`). */
  scope: PermissionScope;
  /** Needed for this one to mean anything; also its parent in the editor's tree. */
  dependsOn?: Permission;
}

const { SYSTEM, ORGANIZATION, PROJECT } = PermissionScope;

/** Ordered by scope, then by tree: the order the editor renders. */
export const PERMISSION_CATALOG: PermissionDefinition[] = [
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
  {
    // Also carries MANAGE_SYSTEM_GRANTS (see IMPLICIT_PERMISSIONS_BY_SCOPE).
    id: Permission.MANAGE_ROLES,
    label: msg`Manage roles, and grant them anywhere`,
    scope: SYSTEM,
  },
  {
    // Hidden at SYSTEM scope (MANAGE_ROLES stands in); this label is for the detail panels.
    id: Permission.MANAGE_SYSTEM_GRANTS,
    label: msg`Grant and revoke roles anywhere`,
    scope: SYSTEM,
  },

  {
    // Belonging to the organization: always conferred at this scope (see IMPLICIT_PERMISSIONS_BY_SCOPE).
    id: Permission.READ_ORGANIZATION,
    label: msg`Member of the organization`,
    broadLabel: msg`See every organization on the instance`,
    scope: ORGANIZATION,
  },
  {
    id: Permission.UPDATE_ORGANIZATION,
    label: msg`Edit the organization`,
    broadLabel: msg`Edit every organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    id: Permission.DELETE_ORGANIZATION,
    label: msg`Delete the organization`,
    broadLabel: msg`Delete every organization`,
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
    id: Permission.CREATE_AGENT,
    label: msg`Create agents`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_AGENTS,
  },
  {
    id: Permission.READ_APP,
    label: msg`Open an agent`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_AGENTS,
  },
  {
    id: Permission.READ_APP_STATS,
    label: msg`View agent statistics`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_AGENTS,
  },
  {
    id: Permission.DELETE_APP,
    label: msg`Delete agents`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_AGENTS,
  },
  {
    // Also stands in for READ_PROJECT at organization scope (see IMPLICIT_PERMISSIONS_BY_SCOPE).
    id: Permission.LIST_PROJECTS_BY_ORGANIZATION,
    label: msg`See and open every project in the organization`,
    // A system role must ask for READ_PROJECT separately.
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
  {
    id: Permission.LIST_ORGANIZATION_MEMBERS,
    label: msg`See who belongs to the organization`,
    broadLabel: msg`See who belongs to any organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.READ_ORGANIZATION,
  },
  {
    // Bounded by the backend to this organization and its projects. Under the member list: managing a list you cannot read is useless.
    id: Permission.MANAGE_ORG_GRANTS,
    label: msg`Grant and revoke roles in the organization`,
    broadLabel: msg`Grant and revoke roles in every organization`,
    scope: ORGANIZATION,
    dependsOn: Permission.LIST_ORGANIZATION_MEMBERS,
  },

  {
    // Belonging to the project: puts it in the holder's list.
    id: Permission.READ_PROJECT,
    label: msg`Member of the project`,
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
  {
    id: Permission.LIST_PROJECT_MEMBERS,
    label: msg`See who works on the project`,
    broadLabel: msg`See who works on any project`,
    scope: PROJECT,
    dependsOn: Permission.READ_PROJECT,
  },
  {
    // The project counterpart of MANAGE_ORG_GRANTS.
    id: Permission.MANAGE_PROJECT_GRANTS,
    label: msg`Grant and revoke roles on the project`,
    broadLabel: msg`Grant and revoke roles on every project`,
    scope: PROJECT,
    dependsOn: Permission.LIST_PROJECT_MEMBERS,
  },

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

const DEFINITION_BY_PERMISSION = new Map<Permission, PermissionDefinition>(
  PERMISSION_CATALOG.map(definition => [definition.id, definition]),
);

export const getPermissionDefinition = (permission: Permission): PermissionDefinition | undefined =>
  DEFINITION_BY_PERMISSION.get(permission);

export const isEditablePermission = (permission: Permission): boolean =>
  DEFINITION_BY_PERMISSION.has(permission);

export const ALL_SCOPES: PermissionScope[] = [SYSTEM, ORGANIZATION, PROJECT];

export const SCOPE_LABELS: Record<PermissionScope, MessageDescriptor> = {
  [PermissionScope.SYSTEM]: msg`System`,
  [PermissionScope.ORGANIZATION]: msg`Organization`,
  [PermissionScope.PROJECT]: msg`Project`,
  [PermissionScope.UNSPECIFIED]: msg({ context: 'feminine', message: 'Unknown' }),
};

export const scopeLabelOf = (scope: PermissionScope): string => i18n._(SCOPE_LABELS[scope]);

/**
 * Outside the catalog, falls back to the humanized enum key. `roleScope` is the
 * scope of the role carrying it: a broader scope gets the plural wording.
 */
export const permissionLabelOf = (permission: Permission, roleScope?: PermissionScope): string => {
  const definition = getPermissionDefinition(permission);
  if (!definition) return humanizePermission(permission);

  const broadened = roleScope !== undefined && roleScope !== definition.scope;
  return i18n._(broadened ? (definition.broadLabel ?? definition.label) : definition.label);
};

/** A role may confer the permissions of its scope and of every broader one. */
const SCOPE_HIERARCHY: Record<PermissionScope, PermissionScope[]> = {
  [PermissionScope.SYSTEM]: [SYSTEM, ORGANIZATION, PROJECT],
  [PermissionScope.ORGANIZATION]: [ORGANIZATION, PROJECT],
  [PermissionScope.PROJECT]: [PROJECT],
  [PermissionScope.UNSPECIFIED]: [],
};

export const getPermissionDefinitionsForScope = (
  scope: PermissionScope,
): PermissionDefinition[] => {
  const allowed = SCOPE_HIERARCHY[scope] ?? [];
  return PERMISSION_CATALOG.filter(definition => allowed.includes(definition.scope));
};

/** Never shown by the editor at a scope, and written on the user's behalf. */
interface ImplicitPermission {
  id: Permission;
  /** Takes its place (adopts its children, ticking it writes `id` too). Absent: always conferred, shown locked. */
  standsIn?: Permission;
}

/**
 * What each scope confers without asking, because it is what the scope already means:
 * - organization: READ_ORGANIZATION is belonging; READ_PROJECT rides on
 *   LIST_PROJECTS_BY_ORGANIZATION (listing every project without opening any is useless).
 * - system: MANAGE_SYSTEM_GRANTS rides on MANAGE_ROLES (who writes roles hands them out).
 */
const IMPLICIT_PERMISSIONS_BY_SCOPE: Partial<Record<PermissionScope, ImplicitPermission[]>> = {
  [PermissionScope.SYSTEM]: [
    { id: Permission.MANAGE_SYSTEM_GRANTS, standsIn: Permission.MANAGE_ROLES },
  ],
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

/** The ticked, locked rows above the tree. */
export const getAlwaysGrantedPermissionsForScope = (scope: PermissionScope): Permission[] =>
  getImplicitAtScope(scope)
    .filter(entry => entry.standsIn === undefined)
    .map(entry => entry.id);

export const isHiddenAtScope = (permission: Permission, scope: PermissionScope): boolean =>
  getImplicitAtScope(scope).some(entry => entry.id === permission);

/** What gets written: the choices, what the scope confers, and what a ticked stand-in carries. */
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

/** What the editor renders. A child of a hidden entry hangs under its stand-in, or becomes a root. */
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

export const getPermissionsForScope = (scope: PermissionScope): Permission[] =>
  getPermissionDefinitionsForScope(scope).map(definition => definition.id);

/** `LIST_JOBS_BY_PIPELINE` → "List jobs by pipeline". Untranslated. */
export const humanizePermission = (permission: Permission): string => {
  const key = Permission[permission] as string | undefined;
  if (!key) return `Permission #${permission}`;
  const words = key.toLowerCase().replace(/_/g, ' ');
  return words.charAt(0).toUpperCase() + words.slice(1);
};
