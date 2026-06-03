import { Permission, PrincipalKind, Scope } from '@/generated/permission.ts';

// The strongly-typed proto enums are numeric. These helpers derive the full
// list of real values (dropping the *_UNSPECIFIED = 0 sentinel) and map a value
// back to its SCREAMING_SNAKE name for display — no string guessing needed.

const realValues = <T extends Record<string, string | number>>(e: T): number[] =>
  Object.values(e).filter((v): v is number => typeof v === 'number' && v !== 0);

/** Every enforceable permission (closed catalog), minus the unspecified sentinel. */
export const ALL_PERMISSIONS = realValues(Permission) as Permission[];

/** Map a permission value to its catalog name, e.g. `RUN_PIPELINE`. */
export const permissionName = (p: Permission): string => Permission[p] ?? String(p);

/** Grantable scope kinds (System / Organization / Project). */
export const ALL_SCOPES = realValues(Scope) as Scope[];

export const scopeName = (s: Scope): string => Scope[s] ?? String(s);

/** Principal kinds for the effective-permissions lookup. */
export const ALL_PRINCIPAL_KINDS = realValues(PrincipalKind) as PrincipalKind[];

export const principalKindName = (k: PrincipalKind): string => PrincipalKind[k] ?? String(k);

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
