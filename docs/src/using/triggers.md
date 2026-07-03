# Triggers

A **trigger** is a stored way to start a pipeline *without* a human clicking
"Run". Each trigger is bound to exactly one pipeline (a pipeline can have several
triggers). When it fires it takes the **same path as a manual run** — one
`runPipeline` check, one job minted, the same dispatch to an agent. A trigger is a
new *source* of runs, never a new execution path.

Scylla ships two source kinds:

- **Cron** — fires on a schedule.
- **Webhook** — fires when an external system POSTs to the trigger's URL.

## How a triggered run is authorized

A triggered run executes as a dedicated **machine principal scoped to the org** —
an App carrying only `runPipeline` — never as the human who created the trigger
and never as a system service. Consequently, **creating a trigger requires that
you can run the pipeline yourself**: you can't launder authority you don't have
into a scheduled run. All trigger operations are gated by `manageTriggers` on the
target pipeline.

The source **kind is immutable** — to switch a trigger from cron to webhook,
delete it and create a new one.

## Cron triggers

A cron trigger fires on a 5-field schedule evaluated in **UTC** (there is no
per-tenant timezone yet):

```
min  hour  day-of-month  month  day-of-week
```

Example — 09:00 UTC on weekdays:

```
0 9 * * 1-5
```

Sub-minute schedules are rejected; the minimum interval is enforced server-side.
The trigger's `next_fire_at` shows the next due occurrence.

## Webhook triggers

A webhook trigger exposes a URL you POST to:

```
POST /webhooks/{trigger_id}      # on the control plane's webhook port (8088)
```

Requests are authenticated by an **HMAC-SHA256 signature of the raw request
body**. When you create a webhook trigger, Scylla generates the signing secret and
returns it **exactly once** (`webhook_secret`) — copy it into the sender's config
immediately, because it is stored encrypted and never shown again. The sender
signs the raw body and sends the hex signature in a header. The header is
configurable: leave it blank for Scylla's default header, or set
`X-Hub-Signature-256` to accept GitHub-style signatures.

The endpoint's responses are deliberately opaque so it never leaks which trigger
ids exist:

| Result | Status | Meaning |
|--------|--------|---------|
| Fired | `202 Accepted` | Signature valid; a job was minted (Pending until an agent picks it up). |
| Duplicate | `200 OK` | A delivery already seen — idempotent replay, ignored. |
| Not found | `404 Not Found` | Unknown trigger id, **or a disabled trigger**. |
| Bad signature | `401 Unauthorized` | HMAC verification failed. |

**Replay dedupe:** if the sender includes a delivery-id header (`X-Scylla-Delivery`
or `X-GitHub-Delivery`), Scylla dedupes repeated deliveries so a retrying sender
doesn't fire the pipeline twice.

## Trigger inputs

A trigger can inject **inputs** into its runs as environment variables (literal,
unmasked, merged after secrets are resolved). Each input is either:

- a **literal** — a constant value, or
- a **JSON pointer** (webhook only) — an [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901)
  path extracting a single value from the webhook payload, e.g. `/after`.

Inputs are allowlisted per trigger — there is no "splat the whole body into the
environment", and an input can never name a secret.

## Managing triggers

- **Enable / disable** a trigger without deleting it. A disabled trigger never
  fires, and a disabled webhook returns the opaque `404`.
- **Fire now** triggers a run immediately, bypassing the schedule and signature
  check — useful for testing. It mints and dispatches a job exactly like a real
  fire and requires `runPipeline`.
- Each trigger records its **last fire** result (`last_fired_at`, `last_status`)
  for observability.
