import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class RevokeGrantUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(id: string) {
    return this.repository.revokeGrant(id);
  }
}
