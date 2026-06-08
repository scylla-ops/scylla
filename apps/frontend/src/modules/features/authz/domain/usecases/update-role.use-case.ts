import type {
  AuthzRepository,
  UpdateRoleInput,
} from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class UpdateRoleUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(input: UpdateRoleInput) {
    return this.repository.updateRole(input);
  }
}
