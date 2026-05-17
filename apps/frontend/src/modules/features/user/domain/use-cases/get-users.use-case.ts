import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';

export class GetUsersUseCase {
  constructor(private readonly _repository: UserRepository) {}

  public async execute(): Promise<ScyllaResult<PaginatedList<User>>> {
    return this._repository.getAll();
  }
}
