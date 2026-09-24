import { msg } from '@lingui/core/macro';

export const jobsMessages = {
  pipelineIdMissing: msg`Pipeline ID is missing`,
  loadError: msg`Unable to load jobs`,
  noJobsFound: msg`No jobs found`,
  noJobsBody: msg`Run your pipeline to create the first job`,

  job: msg`Job`,
  jobs: msg`Jobs`,
  run: msg`Run`,
  runDenied: msg`You don't have permission to run this pipeline.`,
  deleteDenied: msg`You don't have permission to delete jobs.`,
  pipelineIdLabel: msg`Pipeline ID:`,
  refresh: msg`Refresh`,

  status: msg`Status`,
  jobId: msg`Job ID`,
  timeline: msg`Timeline`,
  duration: msg`Duration`,
  created: msg`Created`,
  actions: msg`Actions`,
  copyId: msg`Copy ID`,
  queuedHint: msg`queued — waiting for an agent`,
  orphanedHint: msg`agent disconnected mid-run`,
  view: msg`View`,
  delete: msg`Delete`,
  jobActions: msg`Job actions`,

  deleteJobTitle: msg`Delete Job`,
  deleteJobBody: (jobId: string) =>
    msg`Are you sure you want to delete job ${jobId}? This action cannot be undone.`,

  noNodes: msg`No nodes`,
  nodeLabel: (nodeId: string) => msg`Node ${nodeId}`,
  nodeState: (state: string) => msg`State: ${state}`,
  nodeStarted: (time: string) => msg`Started: ${time}`,
  nodeFinished: (time: string) => msg`Finished: ${time}`,
  nodeDuration: (duration: string) => msg`Duration: ${duration}`,
  groupLabel: (count: number, status: string) => msg`${count} ${status} nodes`,
  groupShare: (count: number, total: number, percent: number) =>
    msg`${count} / ${total} nodes (${percent}%)`,

  jobIdMissing: msg`Job ID is missing`,
  jobNotFound: msg`Job not found`,
  jobLoadError: msg`Unable to load this job`,
  copyJobId: msg`Copy job id`,
  // `context` separates it from the column header: "Créé le" vs "Créé".
  startedPrefix: msg({ context: 'date-prefix', message: 'Started' }),
  finishedPrefix: msg({ context: 'date-prefix', message: 'Finished' }),
  createdPrefix: msg({ context: 'date-prefix', message: 'Created' }),

  logsDenied: msg`You don't have permission to view this job's logs`,
  logs: msg`Logs`,
  streamingLive: msg`Streaming live output`,
  nodeExecutions: msg`Node executions`,
  wholeJob: msg`Whole job`,
  closeLogsFor: (label: string) => msg`Close the logs for ${label}`,
  collapseLogsFor: (label: string) => msg`Collapse the logs for ${label}`,
  expandLogsFor: (label: string) => msg`Expand the logs for ${label}`,

  loading: msg`Loading...`,
  logsError: msg`Error loading logs...`,
};
