import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { PipelineRecord } from '@/generated/pipeline';
import type { PipelineDashboardRepository } from '@/modules/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';

export class GetPipelineStatsUseCase {
    constructor(private readonly pipelineDashboardRepository: PipelineDashboardRepository) {}

    public execute(id: string): Promise<ScyllaResult<PipelineRecord>> {
        return this.pipelineDashboardRepository.getPipelineStatsById(id);
    }
}