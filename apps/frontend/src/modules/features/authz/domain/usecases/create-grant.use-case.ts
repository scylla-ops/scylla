import type {
  AuthzRepository,
  CreateGrantInput,
} from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class CreateGrantUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(input: CreateGrantInput) {
    return this.repository.createGrant(input);
  }
}
