import type { GrantEntity } from '@/modules/features/role/domain/entities/grant.entity.ts';
import type {
  CreateGrantInput,
  PermissionRepository,
} from '@/modules/features/role/domain/repository/permission.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export type { CreateGrantInput };

export class CreateGrantUseCase {
  constructor(private readonly _repository: PermissionRepository) {}

  public execute(input: CreateGrantInput): Promise<ScyllaResult<GrantEntity>> {
    return this._repository.createGrant(input);
  }
}
