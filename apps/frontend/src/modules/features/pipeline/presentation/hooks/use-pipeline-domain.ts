import { useModuleDomain } from '@platform/di';
import type { PipelineModule } from '../../pipeline.module.ts';

/** Typed access to the pipeline module's use cases. */
export const usePipelineDomain = () => useModuleDomain<typeof PipelineModule.domain>('pipeline');
