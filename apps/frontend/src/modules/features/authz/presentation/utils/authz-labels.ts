import { Permission, PrincipalKind, Scope } from '@/generated/permission.ts';

// The strongly-typed proto enums are numeric. These helpers derive the full
// list of real values (dropping the *_UNSPECIFIED = 0 sentinel) and map a value
// back to its SCREAMING_SNAKE name for display — no string guessing needed.

const realValues = <T extends Record<string, string | number>>(e: T): number[] =>
  Object.values(e).filter((v): v is number => typeof v === 'number' && v !== 0);

/** Every enforceable permission (closed catalog), minus the unspecified sentinel. */
export const ALL_PERMISSIONS = realValues(Permission) as Permission[];

/**
 * Turn a proto enum's SCREAMING_SNAKE name into a human label: `RUN_PIPELINE` →
 * "Run pipeline", `LIST_JOBS_BY_ORGANIZATION` → "List jobs by organization". An
 * optional prefix (e.g. `SCOPE_`) is stripped first so `SCOPE_SYSTEM` → "System".
 */
const humanize = (name: string, prefix = ''): string => {
  const s = (prefix && name.startsWith(prefix) ? name.slice(prefix.length) : name)
    .toLowerCase()
    .replace(/_/g, ' ')
    .trim();
  return s.charAt(0).toUpperCase() + s.slice(1);
};

/** Human label for a permission, e.g. `RUN_PIPELINE` → "Run pipeline". */
export const permissionName = (p: Permission): string =>
  Permission[p] ? humanize(Permission[p]) : String(p);

/** Grantable scope kinds (System / Organization / Project). */
export const ALL_SCOPES = realValues(Scope) as Scope[];

export const scopeName = (s: Scope): string =>
  Scope[s] ? humanize(Scope[s], 'SCOPE_') : String(s);

/** Principal kinds for the effective-permissions lookup. */
export const ALL_PRINCIPAL_KINDS = realValues(PrincipalKind) as PrincipalKind[];

export const principalKindName = (k: PrincipalKind): string =>
  PrincipalKind[k] ? humanize(PrincipalKind[k], 'PRINCIPAL_KIND_') : String(k);

/**
 * Pull the most useful message out of a thrown error. The data source wraps
 * failures in a ScyllaError whose `cause` is the underlying gRPC RpcError — that
 * cause carries the real backend message (e.g. an anti-escalation denial), so
 * prefer it for the wireframe's inline error text.
 */
export function errorMessage(err: unknown): string {
  if (err && typeof err === 'object' && 'cause' in err) {
    const cause = (err as { cause?: unknown }).cause;
    if (cause instanceof Error) return cause.message;
  }
  return err instanceof Error ? err.message : String(err);
}
