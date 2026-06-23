import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

export class GetUsersUseCase {
  constructor(private readonly _repository: UserRepository) {}

  public async execute(): Promise<ScyllaResult<PaginatedList<UserEntity>>> {
    return this._repository.getAll();
  }
}
