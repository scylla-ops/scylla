import { msg } from '@lingui/core/macro';

/**
 * Every string the pipeline screens show.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate.
 *
 * Named placeholders where the React original had positional ones: `{0}` is
 * what the macro emits for an expression it cannot name —
 * `getRelativeTime(job.createdAt)` — and a function parameter always has one,
 * so those four msgids changed and their French was carried across by hand.
 * Every message the original *could* name keeps that name, because there the
 * msgid survives and renaming it would have emptied the translation in silence.
 *
 * It sits at `presentation/` rather than under `ui/` because `pipeline.queries.ts`
 * needs one of them: the duplicate mutation names the copy it creates.
 */
export const pipelineMessages = {
  // ── Dashboard ──────────────────────────────────────────────────────────────
  loadError: msg`Unable to load pipelines`,
  noPipelines: msg`No pipeline found`,
  noPipelinesBody: msg`Create your first pipeline to get started`,

  // ── Dashboard header ───────────────────────────────────────────────────────
  pipeline: msg`Pipeline`,
  pipelines: msg`Pipelines`,
  newPipeline: msg`New pipeline`,
  createDenied: msg`You don't have permission to create pipelines.`,
  deleteDenied: msg`You don't have permission to delete pipelines.`,
  members: msg`Members`,
  secrets: msg`Secrets`,

  // ── Table columns ──────────────────────────────────────────────────────────
  status: msg`Status`,
  history: msg`History`,
  lastRun: msg`Last Run`,
  actions: msg`Actions`,
  creation: msg`Creation:`,

  // ── Row actions ────────────────────────────────────────────────────────────
  run: msg`Run`,
  edit: msg`Edit`,
  editPipeline: msg`Edit pipeline`,
  duplicate: msg`Duplicate`,
  viewJobs: msg`View Jobs`,
  triggers: msg`Triggers`,
  /** New: the React compact dropdown trigger carried no accessible name. */
  pipelineActions: msg`Pipeline actions`,

  // ── History strip ──────────────────────────────────────────────────────────
  jobsForbidden: msg`You don't have permission to view this pipeline's jobs`,
  jobsError: msg`Error loading jobs`,
  noJobsYet: msg`No jobs yet`,
  openLastRun: msg`Open the last run`,
  runNumber: (runNumber: number) => msg`Run #${runNumber}`,
  startedAt: (time: string) => msg`Started ${time}`,
  finishedAt: (time: string) => msg`Finished ${time}`,
  durationOf: (duration: string) => msg`Duration: ${duration}`,

  // ── Editor ─────────────────────────────────────────────────────────────────
  selectProjectFirst: msg`Select a project first`,
  create: msg`Create`,
  save: msg`Save`,
  loadPipelineError: msg`Failed to load pipeline`,
  loading: msg`Loading...`,
  invalidJson: msg`Invalid JSON`,
  scripting: msg`Scripting`,
  blueprint: msg`Blueprint`,
  statusDraft: msg`Draft — not created yet`,
  statusSaving: msg`Saving…`,
  statusDirty: msg`Unsaved changes`,
  statusSaved: msg`All changes saved`,
  editDenied: msg`You don't have permission to edit this pipeline.`,

  // ── Blueprint canvas ───────────────────────────────────────────────────────
  addNode: msg`Add Node`,
  deleteNode: msg`Delete node`,
  deleteEdge: msg`Delete edge`,
  unnamedPipeline: msg`Unnamed`,

  // ── Pipeline-name dialog ───────────────────────────────────────────────────
  pipelineName: msg`Pipeline name`,
  pipelineNameDescription: msg`Set the name of the pipeline.`,
  name: msg`Name`,
  namePlaceholder: msg`e.g., my-pipeline`,

  // ── Step dialog ────────────────────────────────────────────────────────────
  addNodeTitle: msg`Add a new node`,
  addNodeDescription: msg`Define a new pipeline step with a unique ID and clear inputs for the command or script it should run.`,
  editNodeTitle: msg`Edit node`,
  editNodeDescription: msg`Update the step details and dependencies.`,
  nodeId: msg`Node ID`,
  nodeIdPlaceholder: msg`e.g., build-step`,
  script: msg`Script`,
  command: msg`Command`,
  commandPlaceholder: msg`e.g., cargo build && cargo test`,
  shell: msg`Shell`,
  argumentsLabel: msg`Arguments`,
  argumentPlaceholder: msg`e.g., --release`,
  addArgument: msg`Add argument`,
  workingDirectory: msg`Working directory`,
  workingDirectoryPlaceholder: msg`e.g., ./services/api or /workspace`,
  environmentVariables: msg`Environment variables`,
  envKeyPlaceholder: msg`KEY`,
  envValuePlaceholder: msg`value`,
  envKindLiteral: msg`Literal`,
  envKindSecret: msg`Secret`,
  /** The kind picker has no visible label — a row of them needs one each. */
  envKindLabel: msg`Value kind`,
  referenceASecret: msg`Reference a secret`,
  secretNamePlaceholder: msg`secret name`,
  addVariable: msg`Add variable`,
  /** New: the trash buttons on argument and variable rows had no name. */
  removeArgument: msg`Remove argument`,
  removeVariable: msg`Remove variable`,
  cancel: msg`Cancel`,

  // ── Duplication ────────────────────────────────────────────────────────────
  copyOf: (name: string) => msg`${name} (copy)`,
};
