# Triggers (cron & webhooks)

A trigger starts a run of one pipeline without a `RunPipeline` call. There are
two kinds:

- a **cron** trigger fires on a schedule, in UTC;
- a **webhook** trigger fires when an external system, such as GitHub, POSTs
  a signed request to its URL.

A pipeline may hold several triggers. A fire creates a job through the same
path as a manual run, so the job waits for a free agent like any other. The
source of truth is `scylla/trigger/v1/trigger.proto` in
[scylla-protos](https://github.com/scylla-ops/scylla-protos).

## Create a trigger

In the web UI, open the pipeline, then its triggers, and click
**New trigger**. Through the API, call `TriggerService.CreateTrigger`.

| Field     | Rules                                                              |
| --------- | ------------------------------------------------------------------ |
| `name`    | Trimmed; 1 to 255 bytes. Unique within the pipeline.               |
| `cron` or `webhook` | Required. The kind cannot change later: delete the trigger and create another one. |
| `inputs`  | At most 128. See [Inputs](#inputs).                                |

Creating or changing a trigger needs `manageTriggers` and `runPipeline` on the
pipeline. The Project Developer role grants both. A new trigger is enabled.

## Cron triggers

The expression has exactly five fields: minute, hour, day of month, month,
day of week. The server evaluates it in **UTC**. A sixth field, for seconds,
is rejected, so the finest step is one minute. The server rejects an
expression that does not parse or has no upcoming occurrence.

| Expression     | Fires                                  |
| -------------- | -------------------------------------- |
| `0 9 * * 1-5`  | 09:00 UTC, Monday to Friday            |
| `*/15 * * * *` | Every 15 minutes                       |
| `30 2 * * *`   | 02:30 UTC every day                    |

The control plane checks for due triggers every 15 seconds, so a run starts
up to about 15 seconds after its minute.

Missed occurrences: when several occurrences were missed, for example while
the control plane was down, the trigger fires **once** and computes the next
occurrence from now. When you re-enable a trigger or change its expression,
the next occurrence is computed from now; the occurrences missed while it was
disabled do not fire.

The web form shows times in the local time of your browser. When the minute
and the hour of the expression are plain numbers, the form converts them to
UTC with the offset of the day you save, and the server stores the UTC
expression. A later daylight saving change therefore moves the run by one
hour in local time.

## Webhook triggers

### URL

A webhook trigger listens on

```
POST <public address>/webhooks/<trigger_id>
```

on the same port as the UI and the API. The trigger shows this URL when the
server has `[webhook].public_base_url` set (see
[Configuration](./configuration.md#webhook)). The sender must be able to
reach that address.

### Signing secret

`CreateTrigger` returns a signing secret of 64 hex characters, **once**. No
later call returns it. Copy it into the configuration of the sender. Changing
the trigger with `UpdateTrigger` keeps the secret. To get a new secret, delete
the trigger and create another one.

The sender computes an HMAC-SHA256 of the raw request body, with the signing
secret as key. It sends the hex digest, with or without a `sha256=` prefix,
in the signature header. The prefix and the hex digits may be in any case.

The signature header is `X-Scylla-Signature-256` by default. Set
`signature_header` to use another one; for GitHub, set
`X-Hub-Signature-256`. A header name is at most 255 bytes of ASCII letters,
digits and ``!#$%&'*+-.^_`|~``.

### Delivery id and duplicates

A delivery id in the `X-Scylla-Delivery` or `X-GitHub-Delivery` header
deduplicates: a second delivery with the same id answers 200 and does not fire
again. Without a delivery id, the digest of the signature is the key, so the
same body with the same signature fires once.

A delivery whose fire fails is not recorded. A retry with the same delivery
id fires.

### Ping

A delivery whose `X-Scylla-Event` or `X-GitHub-Event` header is `ping` answers
200 and does not fire. The server checks the signature first, so a ping with
a wrong secret answers 401. GitHub sends a ping when you create the webhook;
any sender can use one to test the URL and the secret.

### Answers

| Status | Body               | Meaning                                                           |
| ------ | ------------------ | ----------------------------------------------------------------- |
| `202`  | `accepted`         | The trigger fired and a job was created. The job may still wait for an agent. |
| `200`  | `duplicate`        | This delivery id (or signature) was seen before. Nothing fired.   |
| `200`  | `pong`             | A signed ping. Nothing fired.                                     |
| `401`  | `invalid signature` | The signature header is missing or wrong.                       |
| `404`  | `not found`        | No such trigger, the trigger is disabled, or it is not a webhook trigger. |
| `422`  | `invalid payload`  | A value from the payload cannot be used by the run.               |
| `500`  | `internal error`   | Any other failure, for example a `secret_ref` of the pipeline that matches no secret. The server log has the detail. |

### Event types

Only the ping is filtered. **Every other delivery fires the pipeline**, whatever
its event type. If the sender can send several event types, choose the events
on the sender's side, or check them in the pipeline (see the inputs below).

### Test with curl

```sh
SECRET=<signing secret>
URL=http://localhost:8080/webhooks/<trigger_id>
BODY='{"ref":"refs/heads/main","after":"0123abc"}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$SECRET" | sed 's/^.*= //')

curl -i -X POST "$URL" \
  -H "Content-Type: application/json" \
  -H "X-Scylla-Signature-256: sha256=$SIG" \
  -H "X-Scylla-Delivery: test-1" \
  --data "$BODY"
```

The first call answers 202. The same call again answers 200 `duplicate`.

## Inputs

An input adds one environment variable to every node of the run.

| Field          | Rules                                                              |
| -------------- | ------------------------------------------------------------------ |
| `key`          | Unique within the trigger. ASCII letters, digits and `_`; not starting with a digit or with `SCYLLA_`; at most 255 bytes. |
| `literal`      | A fixed value, at most 65536 bytes. Cron and webhook triggers.     |
| `json_pointer` | An [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) pointer into the JSON body of the delivery, for example `/after`. Starts with `/`; each `~` is followed by `0` or `1`; at most 1024 bytes. Webhook triggers only. |

- A `json_pointer` input is skipped when it does not resolve, when the body is
  not JSON, and when the fire has no payload (cron, `FireTriggerNow`).
- A string value arrives as is. Any other value arrives as JSON text, for
  example `42`, `true`, `null` or `{"a":1}`.
- A node env key wins over an input with the same key.
- Inputs are not masked in the logs, and an input cannot refer to a secret.
  Do not put a secret in a literal input; use a `secret_ref` in the pipeline.

## GitHub

This setup runs the pipeline on each push to the repository.

1. In Scylla, create a webhook trigger on the pipeline:
   - signature header: `X-Hub-Signature-256`;
   - inputs, as JSON pointers:

     | Key          | Pointer                    |
     | ------------ | -------------------------- |
     | `GIT_COMMIT` | `/after`                   |
     | `GIT_REF`    | `/ref`                     |
     | `GIT_URL`    | `/repository/clone_url`    |

   Copy the signing secret and the webhook URL.
2. In GitHub, open the repository **Settings** > **Webhooks** > **Add
   webhook**:
   - **Payload URL**: the webhook URL of the trigger;
   - **Content type**: `application/json`. With
     `application/x-www-form-urlencoded`, the body is not JSON and the
     `json_pointer` inputs are skipped;
   - **Secret**: the signing secret;
   - **Which events**: *Just the push event*.
3. GitHub sends a ping. Its delivery shows `200` and `pong` when the URL and
   the secret are right, `401` when the secret is wrong.
4. Push a commit. The delivery shows `202` and a job starts.

The pipeline then reads the commit from its environment, for example in a
script step:

```sh
git clone "$GIT_URL" src
git -C src checkout "$GIT_COMMIT"
```

For a private repository, add a token as a project secret and refer to it
with `secret_ref` (see [Pipelines](./pipelines.md#environment)).

### Useful pointers

Push event (`X-GitHub-Event: push`):

| Pointer                  | Value                                    |
| ------------------------ | ---------------------------------------- |
| `/after`                 | The commit SHA after the push.           |
| `/before`                | The commit SHA before the push.          |
| `/ref`                   | The full ref, for example `refs/heads/main`. |
| `/deleted`               | `true` when the push deleted the ref.    |
| `/repository/full_name`  | `owner/repo`.                            |
| `/repository/clone_url`  | The HTTPS clone URL.                     |
| `/pusher/name`           | The user who pushed.                     |

Pull request event (`X-GitHub-Event: pull_request`):

| Pointer                         | Value                                  |
| ------------------------------- | -------------------------------------- |
| `/action`                       | `opened`, `synchronize`, `closed`, ... |
| `/number`                       | The pull request number.               |
| `/pull_request/head/sha`        | The head commit SHA.                   |
| `/pull_request/head/ref`        | The head branch name.                  |
| `/pull_request/head/repo/clone_url` | The clone URL of the head repository. |
| `/pull_request/base/ref`        | The base branch name.                  |

### Filter in the pipeline

Every delivery fires, including a push to any branch, a branch deletion and
every pull request action. To build only some of them, add a first node that
fails when the delivery is not one to build, or check the values in your
scripts. For example, with
an input `GIT_REF` from `/ref`:

```sh
case "$GIT_REF" in
  refs/heads/main) ;;
  *) echo "skipping $GIT_REF"; exit 1 ;;
esac
```

A non-zero exit fails the job and skips the other nodes. A job that ends as
`FAILED` for this reason is expected.

## Fire now

`FireTriggerNow`, the **Fire now** action in the UI, creates a job as a real
fire does. It ignores the cron schedule and needs no signed delivery. It has
no payload, so `json_pointer` inputs are skipped. It needs `runPipeline` on
the trigger. A disabled trigger fails with `FAILED_PRECONDITION`.

## Enable and disable

`SetTriggerEnabled` turns a trigger on or off. A disabled trigger never fires,
and its webhook URL answers 404, as for an unknown id. It needs
`manageTriggers` on the trigger.

## Last fire

Each fire, from the schedule, a webhook delivery or `FireTriggerNow`, records
the last observation of the trigger: the time, and `succeeded` or `failed`.
`succeeded` means that a job was created; it says nothing about how the job
ran. `failed` carries only the text `error`, with no detail.

The job records its origin: `cron` or `webhook` with the trigger id, and for a
webhook the delivery id when the sender sent one.

## The trigger runner

A fired run does not run as the user who created the trigger. It runs as the
**trigger runner** of the organization: an App named `trigger-runner`, which
holds the Organization Trigger Runner role. That role grants `runPipeline`
only. The App has no secret.

The server creates it at the first `CreateTrigger` in the organization. You
cannot disable it, delete it or add a secret to it, and no other App may take
the name `trigger-runner`. If an App that is not the runner already has this
name, `CreateTrigger` fails with `ALREADY_EXISTS` until that App is deleted.

A run reads the secrets that its pipeline refers to, also when a trigger
starts it.

## Deleting

`DeleteTrigger` removes a trigger; it needs `manageTriggers` on the trigger.
Deleting a pipeline deletes its triggers.
