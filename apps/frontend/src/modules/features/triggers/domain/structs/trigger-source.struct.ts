/**
 * Value objects describing a trigger's source — no identity of their own.
 * The proto `source` oneof (cron | webhook) becomes a discriminated union the
 * UI can switch on, and a trigger input's value becomes a literal | json-pointer
 * pair.
 */

export enum TriggerKind {
  Cron = 'cron',
  Webhook = 'webhook',
  /** The server sent a source arm this build doesn't know. Read-only, never written. */
  Unknown = 'unknown',
}

/** Cron schedule. `expression` is a 5-field cron expression, evaluated in UTC. */
export interface CronSource {
  kind: TriggerKind.Cron;
  expression: string;
}

/** Webhook source. `webhookUrl` is read-only (server-derived); only set on reads. */
export interface WebhookSource {
  kind: TriggerKind.Webhook;
  /** Header carrying the HMAC signature. Empty = the Scylla default header. */
  signatureHeader: string;
  /** Public URL to POST to. Read-only — derived by the server, carried by the webhook arm. */
  webhookUrl: string;
}

/**
 * A source kind newer than this build. Rendered as-is, never edited: we can't
 * round-trip a shape we don't understand.
 */
export interface UnknownSource {
  kind: TriggerKind.Unknown;
}

export type TriggerSource = CronSource | WebhookSource | UnknownSource;

/** A trigger input value: a constant, or a value extracted from a webhook payload. */
export type TriggerInputValue =
  | { kind: 'literal'; value: string }
  | { kind: 'jsonPointer'; value: string };

export interface TriggerInput {
  key: string;
  value: TriggerInputValue;
}

/** Write-side source (no read-only `webhookUrl`) — used by {@link TriggerDraft}. */
export type TriggerSourceDraft =
  | { kind: TriggerKind.Cron; expression: string }
  | { kind: TriggerKind.Webhook; signatureHeader: string };

/** The kinds a user can actually author — {@link TriggerKind.Unknown} is read-only. */
export type TriggerDraftKind = TriggerSourceDraft['kind'];
