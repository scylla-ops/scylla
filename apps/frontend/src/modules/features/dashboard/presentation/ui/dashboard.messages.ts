import { msg } from '@lingui/core/macro';

/**
 * Every string the dashboard shows.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate. The
 * ids below are byte-identical to the ones the React components carried, which
 * is what keeps every French translation attached to its message.
 *
 * Four of them interpolate a **positional** `{0}`, because the original did:
 * `<Trans>Last run {getRelativeTime(…)}</Trans>` gave lingui an expression, not
 * an identifier, and an expression is numbered rather than named. Wrapping the
 * argument in `String(…)` / `Number(…)` keeps it an expression here, so the
 * msgid is the same string and the translation still matches it. A bare
 * identifier would read better and silently mint a new, untranslated message.
 */
export const dashboardMessages = {
  // Page
  loadError: msg`Unable to load dashboard`,
  projects: msg`Projects`,
  pipelines: msg`Pipelines`,
  runs: msg`Runs`,
  successRate: msg`Success rate`,
  successRateRecent: msg`Success rate (recent)`,
  seeAll: msg`See all`,
  noProjects: msg`No projects yet.`,
  noDescription: msg`No description`,
  noProjectAccess: msg`You don't have access to this project's pipelines`,
  allPipelines: msg`All Pipelines`,
  noPipelines: msg`No pipelines yet.`,
  name: msg`Name`,
  project: msg`Project`,
  nodes: msg`Nodes`,
  updated: msg`Updated`,
  dashboard: msg`Dashboard`,

  // Run activity
  runActivity: msg`Run activity`,
  inProgress: (inFlight: number) => msg`${inFlight} in progress`,
  noRunYet: msg`No pipeline has run yet.`,
  completed: msg`Completed`,
  failed: msg`Failed`,
  cancelled: msg`Cancelled`,
  orphaned: msg`Orphaned`,
  lastRun: (relative: string) => msg`Last run ${String(relative)}`,
  overWindow: (windowTotal: number, totalRuns: number) =>
    msg`over the last ${Number(windowTotal)} of ${totalRuns} runs`,
  overAll: (total: number) => msg`over all ${Number(total)} runs`,

  // Agent outcomes chart
  agentOutcomes: msg`Agent Outcomes`,
  filterAll: msg`all`,
  filterCompleted: msg`completed`,
  filterFailed: msg`failed`,
  filterCancelled: msg`cancelled`,
  noFinishedJobs: msg`No finished jobs in this window.`,
  noAgents: msg`No agents found. Connect an agent to see execution history.`,
};
