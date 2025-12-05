import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { PipelineRecord } from '@/generated/pipeline';

export interface PipelineDashboardStore {
    getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineRecord>>;
}