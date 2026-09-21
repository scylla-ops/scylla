import { msg } from '@lingui/core/macro';

/**
 * Every string the triggers screen shows.
 *
 * `lingui extract` does not read `.svelte`, so a message declared inside a
 * component would vanish from the catalogs without failing a single gate. The
 * ids below are byte-identical to the ones the React components carried —
 * placeholder names included, since those are part of the msgid.
 */
export const triggersMessages = {
  // Page
  pipelineIdMissing: msg`Pipeline ID is missing`,
  loadError: msg`Unable to load triggers`,
  noTriggersYet: msg`No triggers yet`,
  noTriggersBody: msg`Add a cron schedule or webhook to start runs automatically.`,
  webhookSecretOf: (name: string) => msg`${name} webhook secret`,
  copySecretOnce: msg`Copy the secret below — it is shown only once.`,
  addSigningSecret: msg`Add this signing secret to your webhook sender's HMAC configuration.`,

  // Header
  trigger: msg`Trigger`,
  triggers: msg`Triggers`,
  newTrigger: msg`New trigger`,
  manageDenied: msg`You don't have permission to manage triggers.`,
  pipelineIdLabel: msg`Pipeline ID:`,

  // Overview
  enabled: msg`enabled`,
  nextScheduledRun: msg`next scheduled run (Local time)`,
  webhookEndpoints: msg`webhook endpoints`,

  // Table columns
  name: msg`Name`,
  source: msg`Source`,
  status: msg`Status`,
  enabledColumn: msg`Enabled`,
  actions: msg`Actions`,
  copyUrl: msg`Copy url`,
  /** New: the React switch was unlabelled, so no test could reach it by name. */
  toggleEnabled: (name: string) => msg`Enable ${name}`,
  /** New, same reason: the compact dropdown trigger carried no accessible name. */
  triggerActions: msg`Trigger actions`,

  // Status cell
  neverFired: msg`Never fired`,
  error: msg`Error`,
  ok: msg`OK`,
  unknown: msg`Unknown`,
  disabledWebhook: msg`disabled — URL returns 404`,
  disabled: msg`disabled`,
  next: msg`next`,
  last: msg`last`,

  // Row actions
  fireNow: msg`Fire now`,
  edit: msg`Edit`,
  delete: msg`Delete`,

  // Form dialog
  editTrigger: msg`Edit trigger`,
  formDescription: msg`A trigger starts a run of this pipeline automatically.`,
  type: msg`Type`,
  typeLocked: msg`Type can't be changed — delete and recreate to switch.`,
  schedule: msg`Schedule`,
  signatureHeader: msg`Signature header`,
  signatureHeaderHint: msg`Header carrying the HMAC signature. Leave empty for the Scylla default.`,
  cancel: msg`Cancel`,
  save: msg`Save`,
  create: msg`Create`,

  // Inputs editor
  inputs: msg`Inputs`,
  addInput: msg`Add input`,
  inputsHint: msg`Optional values injected into the run as environment variables.`,
  literal: msg`Literal`,
  jsonPointer: msg`JSON pointer`,
  /** New: the per-row delete button was an unnamed icon. */
  removeInput: msg`Remove input`,

  // Cron builder
  hourly: msg`Hourly`,
  everyHour: msg`Every hour`,
  daily: msg`Daily`,
  everyDay: msg`Every day`,
  weekly: msg`Weekly`,
  onChosenDays: msg`On chosen days`,
  monthly: msg`Monthly`,
  onADayOfTheMonth: msg`On a day of the month`,
  custom: msg`Custom`,
  writeACron: msg`Write a cron expression`,
  at: msg`at`,
  localTime: msg`local time`,
  atMinute: msg`at minute`,
  onDay: msg`on day`,
  cronHint: msg`5-field cron (min hour day month weekday), evaluated in your local time.`,
  /** New: the hour/minute/day selects were unlabelled. */
  hour: msg`Hour`,
  minute: msg`Minute`,
  dayOfMonth: msg`Day of the month`,
  frequency: msg`Frequency`,
  weekdays: msg`Days of the week`,
};
