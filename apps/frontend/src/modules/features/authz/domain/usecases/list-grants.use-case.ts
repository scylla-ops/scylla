import type { Scope } from '@/generated/permission.ts';
import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class ListGrantsUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(scope?: Scope, scopeId?: string) {
    return this.repository.listGrants(scope, scopeId);
  }
}
