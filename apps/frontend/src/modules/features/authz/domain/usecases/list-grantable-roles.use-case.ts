import type { Scope } from '@/generated/permission.ts';
import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class ListGrantableRolesUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(scope?: Scope) {
    return this.repository.listGrantableRoles(scope);
  }
}
