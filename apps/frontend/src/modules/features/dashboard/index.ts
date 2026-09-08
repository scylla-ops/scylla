/**
 * The organization landing page: projects and pipelines at a glance.
 *
 * A composite view — it owns no data of its own and reads `project`, `pipeline`
 * and `agents` through their public APIs.
 */
export { useOrgOverview, type ProjectAccess } from './presentation/hooks/use-org-overview.ts';
