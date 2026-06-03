import type {
  AuthzRepository,
  CreateRoleInput,
} from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class CreateRoleUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(input: CreateRoleInput) {
    return this.repository.createRole(input);
  }
}
