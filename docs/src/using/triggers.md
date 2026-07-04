# Triggers

A **trigger** starts a pipeline without a human clicking "Run". Each trigger
is bound to exactly one pipeline (a pipeline can have several) and fires
through the **same path as a manual run** — one `runPipeline` check, one job
minted, the same dispatch. Two source kinds ship: **cron** (schedule) and
**webhook** (external POST). The kind is immutable — to switch, delete the
trigger and create a new one.

## How a triggered run is authorized

A triggered run executes as a dedicated org-scoped **machine principal**
carrying only `runPipeline` — never as the human who created the trigger.
Consequently, **creating a trigger requires that you can run the pipeline
yourself**: you can't launder authority you don't have into a scheduled run.
All trigger operations are gated by `manageTriggers` on the target pipeline.

## Cron triggers

A 5-field schedule (`min hour day-of-month month day-of-week`) evaluated in
**UTC** (no per-tenant timezone yet). Example — 09:00 UTC on weekdays:

```
0 9 * * 1-5
```

Sub-minute schedules are rejected server-side. `next_fire_at` shows the next
due occurrence.

## Webhook triggers

A webhook trigger exposes `POST /webhooks/{trigger_id}` on the control plane's
webhook port (`8088`). Requests are authenticated by an **HMAC-SHA256
signature of the raw request body**. Scylla generates the signing secret at
creation and returns it **exactly once** (`webhook_secret`) — copy it into the
sender's config immediately; it is stored encrypted and never shown again. The
sender sends the hex signature in a header: leave the header name blank for
Scylla's default, or set `X-Hub-Signature-256` to accept GitHub-style
signatures.

Responses are deliberately opaque so the endpoint never leaks which trigger
ids exist:

| Result | Status | Meaning |
|--------|--------|---------|
| Fired | `202 Accepted` | Signature valid; a job was minted (Pending until an agent picks it up). |
| Duplicate | `200 OK` | A delivery already seen — idempotent replay, ignored. |
| Not found | `404 Not Found` | Unknown trigger id, **or a disabled trigger**. |
| Bad signature | `401 Unauthorized` | HMAC verification failed. |

**Replay dedupe:** if the sender includes a delivery-id header
(`X-Scylla-Delivery` or `X-GitHub-Delivery`), repeated deliveries are deduped
so a retrying sender doesn't fire the pipeline twice.

## Trigger inputs

A trigger can inject **inputs** into its runs as environment variables
(literal, unmasked, merged after secrets are resolved). Each input is either:

- a **literal** — a constant value, or
- a **JSON pointer** (webhook only) — an
  [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) path extracting a single
  value from the webhook payload, e.g. `/after`.

Inputs are allowlisted per trigger — there is no "splat the whole body into
the environment", and an input can never name a secret.

## Managing triggers

- **Enable / disable** without deleting. A disabled trigger never fires, and a
  disabled webhook returns the opaque `404`.
- **Fire now** runs immediately, bypassing schedule and signature check —
  useful for testing. It mints and dispatches a real job and requires
  `runPipeline`.
- Each trigger records its last fire (`last_fired_at`, `last_status`).
