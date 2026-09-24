import { msg } from '@lingui/core/macro';

/** Keep the positional `{0}` (with `String(…)`/`Number(…)`): it is part of the msgid. */
export const dashboardMessages = {
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

  agentOutcomes: msg`Agent Outcomes`,
  filterAll: msg`all`,
  filterCompleted: msg`completed`,
  filterFailed: msg`failed`,
  filterCancelled: msg`cancelled`,
  noFinishedJobs: msg`No finished jobs in this window.`,
  noAgents: msg`No agents found. Connect an agent to see execution history.`,
};
