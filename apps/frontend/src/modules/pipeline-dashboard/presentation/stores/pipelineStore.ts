import type { PipelineRecord } from '@/generated/pipeline';
import type { ScyllaError } from '@/modules/core/domain/ScyllaResult';

export class PipelineStore {
  private pipeline: PipelineRecord | null = null;
  private loading = false;
  private error: string | ScyllaError = "";

  constructor() {}

  setPipeline(pipeline: PipelineRecord) {
    this.pipeline = pipeline;
  }

  setLoading(loading: boolean) {
    this.loading = loading;
  }

  setError(error: string | ScyllaError) {
    this.error = error;
  }

  get getPipeline() {
    return this.pipeline;
  }

  get isLoading() {
    return this.loading;
  }

  get getError() {
    return this.error;
  }
}