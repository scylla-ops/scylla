import type { PrincipalKind } from '@/generated/permission.ts';
import type { AuthzRepository } from '@/modules/features/authz/domain/repository/authz.repository.ts';

export class GetEffectivePermissionsUseCase {
  constructor(private readonly repository: AuthzRepository) {}

  public execute(principalKind: PrincipalKind, principalId: string) {
    return this.repository.getEffectivePermissions(principalKind, principalId);
  }
}
