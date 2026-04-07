import type { PipelineCreationRemoteDataSource } from '@/modules/features/pipeline-creation/infrastructure/repository/data-sources/PipelineCreationRemoteDataSource.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PipelineCreationRepository } from '@/modules/features/pipeline-creation/domain/repository/PipelineCreationRepository.ts';

export class PipelineCreationRepositoryImpl implements PipelineCreationRepository {
  constructor(private readonly remoteDataSource: PipelineCreationRemoteDataSource) {}
  public async createPipeline(content: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.createPipeline(content);
  }
}
