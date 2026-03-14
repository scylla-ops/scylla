import type { PipelineResponse } from '@/generated/pipeline.ts';
import type { ScyllaError } from '@core/utils/ScyllaResult.ts';

export class PipelineStore {
  private pipeline: PipelineResponse | null = null;
  private loading = false;
  private error: string | ScyllaError = '';

  constructor() {}

  setPipeline(pipeline: PipelineResponse) {
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
