import type { PipelineDashboardRepository } from '@/modules/features/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';

export class DeletePipeline {
  constructor(private readonly _repository: PipelineDashboardRepository) {}

  public execute = (id: string) => this._repository.deleteById(id);
}
