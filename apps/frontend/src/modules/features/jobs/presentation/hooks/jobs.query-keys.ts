/**
 * Jobs of one pipeline.
 *
 * Lives in `jobs` rather than next to the pipeline dashboard that also reads it:
 * it is this module's cache that gets invalidated, and having pipeline own the
 * key is what used to make the two modules mutually dependent.
 */
export const JOBS_QUERY_KEY = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;
