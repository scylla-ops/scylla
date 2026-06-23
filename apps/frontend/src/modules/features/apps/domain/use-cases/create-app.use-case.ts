import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { CreatedApp } from '@/modules/features/apps/domain/structs/app.struct.ts';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';

/** Use case: create an app. Returns the one-time secret. */
export class CreateAppUseCase {
  constructor(private readonly repository: AppsRepository) {}

  execute(organizationId: string, name: string): Promise<ScyllaResult<CreatedApp>> {
    return this.repository.createApp(organizationId, name);
  }
}
