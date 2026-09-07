import type { PermissionRepository } from '@/modules/features/permission/domain/repository/permission.repository.ts';
import type {
  PermissionScope,
  PrincipalEntity,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface RevokeAllAccessInput {
  principal: PrincipalEntity;
  scope: PermissionScope;
  /** The org/project id to clear; empty for SYSTEM scope. */
  scopeId: string;
}

/**
 * Strips a principal of every grant at a scope and beneath it, answering how
 * many were removed. Used to remove someone from an organization: membership is
 * derived from grants, so clearing them all is what removing means.
 */
export class RevokeAllAccessUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute({
    principal,
    scope,
    scopeId,
  }: RevokeAllAccessInput): Promise<ScyllaResult<number>> {
    return this._repository.revokeAllAccess(principal, scope, scopeId);
  }
}
