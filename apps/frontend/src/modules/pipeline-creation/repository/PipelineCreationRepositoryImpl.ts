import type { PipelineCreationRemoteDataSource } from '@/modules/pipeline-creation/repository/dataSources/PipelineCreationRemoteDataSource.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { PipelineCreationRepository } from '@/modules/pipeline-creation/domain/repository/PipelineCreationRepository.ts';

export class PipelineCreationRepositoryImpl implements PipelineCreationRepository {
  constructor(private readonly remoteStore: PipelineCreationRemoteDataSource) {}
  public async createPipeline(content: string): Promise<ScyllaResult<void>> {
    return this.remoteStore.createPipeline(content);
  }
}
