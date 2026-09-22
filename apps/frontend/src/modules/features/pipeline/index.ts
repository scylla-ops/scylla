/**
 * Pipelines: their definition, their editor, and the metadata the overviews read.
 *
 * The public API of the module. Organization-wide reads live here rather than
 * in the dashboard that needs them, so a pipeline query is never rebuilt
 * against `pipelineRepository` by a module that does not own it.
 *
 * `useOrganizationPipelines` is gone with the rest of the React half (Phase 5).
 * Its replacement, `pipelineQueries.byOrganization`, has been here since Phase 4
 * — a `queryOptions` object with no framework in it — so the migration changed
 * nothing for `dashboard`, which was already reading it.
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
} from './presentation/pipelines.query-keys.ts';
export { asPipelineFeed, pipelineQueries } from './presentation/pipeline.queries.ts';
