/**
 * Pipelines: their definition, their editor, and the metadata the overviews read.
 *
 * The public API of the module. Organization-wide reads live here rather than
 * in the dashboard that needs them, so a pipeline query is never rebuilt
 * against `pipelineRepository` by a module that does not own it.
 */
export type { PipelineEntity } from './domain/entities/pipeline.entity.ts';
export type {
  PipelineMetadata,
  PipelineStep,
  PipelineIdentity,
} from './domain/structs/pipeline.struct.ts';
export {
  PIPELINES_QUERY_KEY,
  ORGANIZATION_PIPELINES_QUERY_KEY,
  PIPELINES_QUERY_ROOT,
} from './presentation/hooks/pipelines.query-keys.ts';
export { useOrganizationPipelines } from './presentation/hooks/use-organization-pipelines.ts';
