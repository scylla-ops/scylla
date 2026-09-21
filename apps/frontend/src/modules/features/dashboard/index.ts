/**
 * The organization landing page: projects and pipelines at a glance.
 *
 * A composite view — it owns no data of its own and reads `project`, `pipeline`,
 * `jobs` and `agents` through their public APIs.
 *
 * Nothing outside imports this today; the barrel exists so that the day
 * something does, it finds a door rather than reaching into the module. What it
 * does **not** export is the page: a `.svelte` component re-exported here could
 * not be dropped by Rollup, and would drag bits-ui into the chunk of whoever
 * imported the barrel for a type.
 */
export { createOrgOverview, type OrgOverview, type ProjectAccess, type PipelineWithProject } from './presentation/org-overview.state.svelte.ts';
