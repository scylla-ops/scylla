import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { PipelineDashboardStore } from '@/modules/pipeline-dashboard/repository/store/PipelineDashboardStore.ts';
import { PipelineClient } from '@/generated/pipeline.client.ts';
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import type { PipelineRecord } from '@/generated/pipeline';

export class PipelineDashboardStoreImpl implements PipelineDashboardStore {
    private readonly _pipelineClient: PipelineClient;

    constructor() {
        const apiUrl = import.meta.env.VITE_API_URL ?? '';
        const transport = new GrpcWebFetchTransport({
            baseUrl: apiUrl,
            format: 'binary',
        });
        this._pipelineClient = new PipelineClient(transport);
    }

    async getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineRecord>> {
        try {
            const { response } = await this._pipelineClient.getPipeline({pipelineId: id});
            return { ok: true, value: response };
        } catch (error) {
            return { ok: false, error: {message: `Failed to fetch pipeline for id: ${id}` + error }};
        }
    }
}