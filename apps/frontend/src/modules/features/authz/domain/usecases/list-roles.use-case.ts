import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class ListRolesUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute() {
    return this.repository.listRoles();
  }
}
