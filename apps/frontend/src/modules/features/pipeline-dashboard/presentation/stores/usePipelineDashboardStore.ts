import { create } from 'zustand';
import type { PipelineResponse } from '@/generated/pipeline.ts';
import type { ScyllaError } from '@core/utils/ScyllaResult.ts';

interface PipelineDashboardState {
  pipelines: PipelineResponse[];
  loading: boolean;
  error: string | ScyllaError;
  setPipelines: (pipelines: PipelineResponse[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | ScyllaError) => void;
  reset: () => void;
}

export const usePipelineDashboardStore = create<PipelineDashboardState>(set => ({
  pipelines: [],
  loading: false,
  error: '',

  setPipelines: (pipelines: PipelineResponse[]) => {
    set({ pipelines });
  },

  setLoading: (loading: boolean) => {
    set({ loading });
  },

  setError: (error: string | ScyllaError) => {
    set({ error });
  },

  reset: () => {
    set({
      pipelines: [],
      loading: false,
      error: '',
    });
  },
}));
