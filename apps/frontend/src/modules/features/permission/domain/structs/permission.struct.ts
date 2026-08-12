/**
 * Mirrors the wire `ScopeKind`. `UNSPECIFIED` doubles as "the backend named a
 * scope this build does not know about".
 */
export enum PermissionScope {
  UNSPECIFIED = 0,
  SYSTEM = 1,
  ORGANIZATION = 2,
  PROJECT = 3,
}

/**
 * Who a grant is for. `UNSPECIFIED` doubles as "the backend named a principal
 * kind this build does not know about".
 */
export enum PrincipalKind {
  UNSPECIFIED = 0,
  USER = 1,
  APP = 2,
}

/** Mirrors the wire `RoleKind`: what a builtin role is for. Descriptive only. */
export enum RoleKind {
  UNSPECIFIED = 0,
  ADMIN = 1,
  AGENT = 2,
  /** A human working inside the scope, short of administering it. */
  MEMBER = 3,
}

/** A grant holder: a user or an app, identified by its plain string id. */
export interface PrincipalEntity {
  kind: PrincipalKind;
  /** Empty when `kind` is `UNSPECIFIED`. */
  id: string;
}

/**
 * Write-side access: what the caller wants a role to confer. Only the two arms
 * this build can actually build are representable.
 */
export type AccessSpec =
  | { kind: 'fullControl' }
  | { kind: 'restricted'; permissions: Permission[] };

/**
 * Read-side access. `unknown` means the backend sent an access arm newer than
 * this build — surface it, never read it as "holds no permission".
 */
export type AccessEntity = AccessSpec | { kind: 'unknown' };

export enum Permission {
  UNSPECIFIED = 0,
  CREATE_USER = 1,
  READ_USER = 2,
  UPDATE_USER = 3,
  DELETE_USER = 4,
  LIST_USERS = 5,
  CREATE_ORGANIZATION = 6,
  READ_ORGANIZATION = 7,
  UPDATE_ORGANIZATION = 8,
  DELETE_ORGANIZATION = 9,
  LIST_ORGANIZATIONS = 10,
  LIST_ORGANIZATION_MEMBERS = 11,
  ADD_ORGANIZATION_MEMBER = 12,
  REMOVE_ORGANIZATION_MEMBER = 13,
  MANAGE_INVITATIONS = 14,
  LIST_USER_ORGANIZATIONS = 15,
  CREATE_PROJECT = 16,
  READ_PROJECT = 17,
  UPDATE_PROJECT = 18,
  DELETE_PROJECT = 19,
  LIST_PROJECTS = 20,
  LIST_PROJECTS_BY_ORGANIZATION = 21,
  LIST_PROJECT_MEMBERS = 22,
  ADD_PROJECT_MEMBER = 23,
  REMOVE_PROJECT_MEMBER = 24,
  LIST_USER_PROJECTS = 25,
  CREATE_PIPELINE = 26,
  READ_PIPELINE = 27,
  UPDATE_PIPELINE = 28,
  DELETE_PIPELINE = 29,
  RUN_PIPELINE = 30,
  EXECUTE_JOB = 31,
  LIST_PIPELINES = 32,
  LIST_PIPELINES_BY_PROJECT = 33,
  LIST_PIPELINES_BY_ORGANIZATION = 34,
  CREATE_JOB = 35,
  READ_JOB = 36,
  UPDATE_JOB = 37,
  DELETE_JOB = 38,
  LIST_JOBS = 39,
  LIST_JOBS_BY_PIPELINE = 40,
  LIST_JOBS_BY_PROJECT = 41,
  LIST_JOBS_BY_ORGANIZATION = 42,

  READ_JOB_LOGS = 43,
  WRITE_JOB_LOGS = 44,
  WRITE_JOB_STATUS = 45,
  APPEND_JOB_LOG = 46,
  CREATE_APP = 47,
  READ_APP = 48,
  READ_APP_STATS = 49,
  DELETE_APP = 50,
  LIST_APPS_BY_ORGANIZATION = 51,
  CREATE_AGENT = 52,
  LIST_AGENTS = 53,
  MANAGE_SYSTEM_GRANTS = 54,
  MANAGE_ORG_GRANTS = 55,
  MANAGE_PROJECT_GRANTS = 56,
  // 57 was MANAGE_POLICIES, dropped with the runtime Cedar policy escape hatch.
  MANAGE_ROLES = 58,
  CREATE_SECRET = 59,
  LIST_SECRETS = 60,
  DELETE_SECRET = 61,
  MANAGE_TRIGGERS = 62,
}
