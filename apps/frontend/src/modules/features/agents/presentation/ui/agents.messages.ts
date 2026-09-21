import { msg } from '@lingui/core/macro';

/**
 * Every string the agents screens show.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate. The
 * ids below are byte-identical to the ones the React components carried —
 * placeholder names included, since those are part of the msgid.
 */
export const agentsMessages = {
  // List page
  agent: msg`Agent`,
  agents: msg`Agents`,
  newAgent: msg`New Agent`,
  createDenied: msg`You don't have permission to create agents.`,
  loadError: msg`Error loading agents`,
  online: msg`online`,
  offline: msg`offline`,
  noAgentsConnected: msg`No agents connected`,
  noAgentsBody: msg`An agent runs jobs. Create one to get credentials, then start the agent — it'll pull jobs and report back.`,
  createFirstAgent: msg`Create your first agent`,
  getCredentials: msg`Get credentials to connect an agent.`,
  createAndReveal: msg`Create & reveal secret →`,

  // Reveal dialog
  agentIsReady: (name: string) => msg`${name} is ready`,
  twoSteps: msg`Two steps and it picks up jobs.`,
  secretForAgent: msg`Secret for the agent`,
  startYourAgent: msg`Start your agent`,

  // Delete confirmation
  deleteAgentTitle: msg`Delete agent?`,
  deleteAgentBody: msg`This revokes the agent's grants and disconnects it. Cannot be undone.`,
  delete: msg`Delete`,

  // Card
  copyId: msg`Copy id`,
  created: msg`created`,
  seen: msg`seen`,
  down: msg`down`,
  neverConnected: msg`never connected`,
  /**
   * New, and deliberately: both icon-only controls on the React card had no
   * accessible name, so nothing could reach them but a CSS selector.
   */
  agentActions: msg`Agent actions`,
  deleteAgent: msg`Delete agent`,

  // Id link
  copyAgentId: msg`Copy agent id`,

  // Details page
  detailsLoadError: msg`Error loading agent`,
  notFound: msg`Agent not found`,
  agentId: msg`Agent ID`,
  activeLabel: msg`Active`,
  active: msg`active`,
  inactive: msg`inactive`,
  createdOn: msg({ context: 'date-prefix', message: 'Created' }),
  updatedOn: msg({ context: 'date-prefix', message: 'Updated' }),
  jobStats: msg`Job stats`,
  since: msg`since`,
  lastRun: msg`last run`,
  runThisAgent: msg`Run this agent`,
  connectsAs: msg`connects as this agent's app id`,

  // Outcomes chart
  outcomesLast: msg`outcomes · last`,
  noFinishedJobs: msg`No finished jobs in this window.`,
  runAPipeline: msg`Run a pipeline on this agent and the chart fills up.`,
  /** `title` on a bar: a plain string, so the four values are placeholders. */
  bucketSummary: (day: string, completed: number, failed: number, cancelled: number) =>
    msg`${day}: ${completed} completed, ${failed} failed, ${cancelled} cancelled`,
  runs: msg`runs`,
  completedCount: (completed: number) => msg`completed ${completed}`,
  failedCount: (failed: number) => msg`failed ${failed}`,
  cancelledCount: (cancelled: number) => msg`cancelled ${cancelled}`,
  completed: msg`completed`,
  failed: msg`failed`,
  cancelled: msg`cancelled`,
  finished: msg`finished`,

  // No-agents banner (consumed by `jobs`)
  jobsQueuedCheckAgents: msg`Jobs are queued — check that your agents are connected.`,
  noAgentConnected: msg`No agent connected — queued jobs are waiting for one.`,
  setUpAnAgent: msg`Set up an agent`,

  // Preview components nobody mounts yet — see AGENTS.md
  logs: msg`Logs`,
  logsNotWired: msg`preview · agent log stream not wired yet`,
  sample: msg`sample`,
  comingSoon: msg`Coming soon`,
  filter: msg`filter`,
  openFullLogs: msg`open full logs`,
  lines: msg`lines`,
  running: msg`running`,
  pending: msg`pending`,
  idleNoJobs: msg`idle — no jobs`,
  queueEmpty: msg`queue empty`,
  waiting: msg`waiting`,
};
