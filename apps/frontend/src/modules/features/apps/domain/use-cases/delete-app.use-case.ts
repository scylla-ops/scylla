import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: delete an app (revokes its grants, disconnects its worker). */
export class DeleteAppUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(appId: string): Promise<ScyllaResult<boolean>> {
    return this.repository.deleteApp(appId);
  }
}
