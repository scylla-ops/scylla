import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class DeleteRoleUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(id: string) {
    return this.repository.deleteRole(id);
  }
}
