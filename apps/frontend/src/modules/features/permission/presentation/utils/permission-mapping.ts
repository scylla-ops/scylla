import {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * Which permissions a role may hold, per the scope it is bound to.
 *
 * Scopes nest broad → narrow (SYSTEM ⊃ ORGANIZATION ⊃ PROJECT): a role at a
 * given scope can hold every permission of its own level plus every narrower
 * level's. The scope only changes the *reach* of the same permission — e.g.
 * `UPDATE_ORGANIZATION` granted at SYSTEM can edit every organization, granted at
 * ORGANIZATION only the organization(s) the grant targets.
 *
 * So each permission is declared **once**, at the narrowest level where it is
 * coherent, and the per-scope sets are composed from those levels — a permission
 * shared by several scopes is never repeated.
 */

/** Permissions first addressable on a single project. */
const PROJECT_LEVEL: Permission[] = [
  // Project
  Permission.UPDATE_PROJECT,
  Permission.DELETE_PROJECT,
  Permission.MANAGE_PROJECT_GRANTS,
  // Pipelines
  Permission.CREATE_PIPELINE,
  Permission.READ_PIPELINE,
  Permission.UPDATE_PIPELINE,
  Permission.DELETE_PIPELINE,
  Permission.RUN_PIPELINE,
  Permission.LIST_PIPELINES_BY_PROJECT,
  // Jobs
  Permission.CREATE_JOB,
  Permission.READ_JOB,
  Permission.UPDATE_JOB,
  Permission.DELETE_JOB,
  Permission.EXECUTE_JOB,
  Permission.LIST_JOBS_BY_PIPELINE,
  Permission.LIST_JOBS_BY_PROJECT,
  Permission.READ_JOB_LOGS,
  Permission.WRITE_JOB_LOGS,
  Permission.WRITE_JOB_STATUS,
  Permission.APPEND_JOB_LOG,
];

/** Permissions first addressable on a single organization. */
const ORGANIZATION_LEVEL: Permission[] = [
  Permission.UPDATE_ORGANIZATION,
  Permission.DELETE_ORGANIZATION,
  Permission.ADD_ORGANIZATION_MEMBER,
  Permission.REMOVE_ORGANIZATION_MEMBER,
  Permission.MANAGE_ORG_GRANTS,
  Permission.CREATE_PROJECT,
  // Cross-project rollups within the organization
  Permission.LIST_PIPELINES_BY_ORGANIZATION,
  Permission.LIST_JOBS_BY_ORGANIZATION,
];

/** Permissions that only make sense system-wide. */
const SYSTEM_LEVEL: Permission[] = [
  Permission.CREATE_ORGANIZATION,
  Permission.CREATE_USER,
  Permission.DELETE_USER,
  Permission.CREATE_AGENT,
  Permission.MANAGE_ROLES,
  Permission.MANAGE_SYSTEM_GRANTS,
  // Global rollups across every tenant
  Permission.LIST_PIPELINES,
  Permission.LIST_JOBS,
];

/**
 * A scope's permissions = its own level plus every narrower level, ordered
 * broad → narrow. SYSTEM sees all three levels, ORGANIZATION the organization +
 * project levels, PROJECT only its own.
 */
export const PERMISSIONS_BY_SCOPE: Map<PermissionScope, Permission[]> = new Map([
  [PermissionScope.SYSTEM, [...SYSTEM_LEVEL, ...ORGANIZATION_LEVEL, ...PROJECT_LEVEL]],
  [PermissionScope.ORGANIZATION, [...ORGANIZATION_LEVEL, ...PROJECT_LEVEL]],
  [PermissionScope.PROJECT, [...PROJECT_LEVEL]],
]);

/** The permissions grantable in a role bound to `scope` (empty if unknown). */
export const permissionsForScope = (scope: PermissionScope): Permission[] =>
  PERMISSIONS_BY_SCOPE.get(scope) ?? [];
