# Pipelines

A pipeline is a directed acyclic graph (DAG) of nodes. Each node runs one
step on an agent. Running a pipeline creates a job. This page is the reference
of the pipeline format. The source of truth is
`scylla/pipeline/v1/pipeline.proto` and `scylla/exec/v1/step.proto` in
[scylla-protos](https://github.com/scylla-ops/scylla-protos).

## Pipeline

| Field        | Rules                                                     |
| ------------ | --------------------------------------------------------- |
| `project_id` | The project that holds the pipeline.                      |
| `name`       | Trimmed; 1 to 255 bytes. Not unique within the project.   |
| `nodes`      | 1 to 256 nodes. Node ids are unique within the list.      |

`UpdatePipeline` changes only what the request carries. An absent `name` is
kept. An empty `nodes` list keeps the nodes; a non-empty list replaces the
whole list. A job created before the update runs the nodes it was created
with.

## Node

| Field         | Rules                                                                                  |
| ------------- | -------------------------------------------------------------------------------------- |
| `node_id`     | 1 to 128 characters: lowercase ASCII letters, digits, `-` and `_`.                     |
| `deps`        | Ids of other nodes of the same pipeline. No self, no duplicate, no unknown id.         |
| `working_dir` | Relative to the job workspace. See [Working directory](#working-directory).           |
| `env`         | At most 128 entries. Keys are unique within the node. See [Environment](#environment). |
| `exec` or `script` | Required. Exactly one. See [Steps](#steps).                                       |

### Dependencies

A node with no `deps` starts first. The agent starts a node when each of its
deps has completed, so independent nodes run at the same time. A dependency
cycle fails with `FAILED_PRECONDITION` when you create or update the pipeline.

### Working directory

- Trimmed. `""` means the root of the job workspace.
- No leading `/` or `\`, no `..` segment. At most 4096 bytes.
- The agent creates the directory when it is missing.
- The agent refuses a directory that resolves outside the workspace, for
  example through a symlink. The node then fails.

### Environment

Each entry has a `key` and exactly one source: `value` or `secret_ref`.

| Field        | Rules                                                                                                 |
| ------------ | ----------------------------------------------------------------------------------------------------- |
| `key`        | Trimmed. Starts with an ASCII letter or `_`. ASCII letters, digits and `_` only. At most 255 bytes. The prefix `SCYLLA_` is reserved. |
| `value`      | Used as is. At most 65536 bytes.                                                                       |
| `secret_ref` | The **name** of a secret of the pipeline's project, not its id. Trimmed; 1 to 128 bytes of ASCII letters, digits, `-`, `_` and `.`. |

`CreatePipeline` and `UpdatePipeline` check the format of a `secret_ref` only.
`RunPipeline` resolves it. A name that matches no secret of the project fails
the run with `NOT_FOUND`, and no job is stored. If you delete a secret, the
pipelines that refer to its name keep the reference, and their next run fails.

The agent does not pass its own environment to a step. A step process gets:

1. `PATH`, `HOME`, `LANG` and `LC_ALL` from the agent process, when they are
   set, and `TERM=dumb`.
2. The node `env`, and the trigger inputs (see [Triggers](./triggers.md)). A
   node key wins over an input with the same key. A node key also replaces a
   variable of step 1, for example `PATH`.
3. Last, so that a node cannot change them: `CI=true`,
   `SCYLLA_WORKSPACE` (the absolute path of the job workspace),
   `SCYLLA_JOB_ID` and `SCYLLA_NODE_ID`.

## Steps

A step runs in the working directory and the environment of its node. Exit
status 0 completes the node. Any other status, a death by signal or a failed
spawn fails the node. The agent writes a spawn error to the stderr log of the
node. When the step process exits, or when the agent stops the node, the agent
kills every process of the step's process group. A process started in the
background does not outlive its node.

### `exec`: one process, no shell

| Field     | Rules                                                     |
| --------- | --------------------------------------------------------- |
| `command` | Trimmed; 1 to 4096 bytes. Not blank.                      |
| `args`    | At most 256 entries, each at most 65536 bytes. May be empty. |

The agent passes `command` and `args` to the operating system as the argv
vector. It does not split words, quote or expand. `$VAR`, globs and pipes are
literal text. The agent looks a bare command name up on its own `PATH` first,
then on a `PATH` set in the node env. A command that contains `/` is used as
given.

### `script`: a shell script

| Field    | Rules                                                       |
| -------- | ----------------------------------------------------------- |
| `script` | Not blank. At most 1048576 bytes (1 MiB). May span many lines. |
| `shell`  | `SHELL_SH` (the default) or `SHELL_BASH`.                   |

The agent writes the script to `.scylla/<node_id>.sh` under the job workspace
and runs that file:

| `shell`      | Command                                                   |
| ------------ | --------------------------------------------------------- |
| `SHELL_SH`   | `sh -e <script file>`                                     |
| `SHELL_BASH` | `bash --noprofile --norc -o pipefail -e <script file>`    |

Both shells run with `-e`: a failing command ends the script with the exit
status of that command. Only `SHELL_BASH` adds `pipefail`. Under `SHELL_SH`, a
pipeline reports the status of its last command, so `make test | tee out.log`
succeeds when `make test` fails. Use `SHELL_BASH` when you need pipefail.

`SHELL_UNSPECIFIED`, and any unknown value, runs as `SHELL_SH`. The shell is
looked up as an `exec` command is. A `PATH` in the node env applies to the
processes that the script starts.

## Job workspace

Each job gets its own directory on the agent: `<workspace root>/<job_id>`. All
nodes of the job share it, so files that one node writes are there for the
nodes that depend on it. The agent removes the directory when the job ends,
unless it runs with `--keep-workspace`. See
[Running an agent](./running-an-agent.md#workspace).

Nothing else is shared between jobs. A job starts in an empty workspace, so
each job fetches its sources itself.

## Failure and cancellation

When a node fails:

1. The agent stops the nodes that are running. It sends `SIGTERM` to their
   process groups, then `SIGKILL` after 5 seconds.
2. These nodes and the nodes that did not start end as `SKIPPED`.
3. The job ends as `FAILED`.

`JobService.CancelJob` ends a pending or running job as `CANCELLED`. Its nodes
that have not finished end as `CANCELLED`, and the agent stops them.

## Job and node outcomes

A job is pending, running or terminal. A terminal job has one outcome:

| Job outcome  | Meaning                                                                                  |
| ------------ | ---------------------------------------------------------------------------------------- |
| `COMPLETED`  | Every node completed.                                                                     |
| `FAILED`     | A node failed, or the agent could not run the job or stopped it when it shut down, or the control plane could not dispatch it (a `secret_ref` no longer resolves, or the resolved job is too large). |
| `CANCELLED`  | `JobService.CancelJob` ended the job.                                                    |
| `ORPHANED`   | The agent stayed disconnected for 60 seconds after its last contact, or reconnected without the job, before it reported a result. |

A finished node has one outcome:

| Node outcome | Meaning                                                    |
| ------------ | ---------------------------------------------------------- |
| `COMPLETED`  | The step exited with status 0.                             |
| `FAILED`     | The step failed.                                           |
| `CANCELLED`  | The job ended before the node finished.                    |
| `SKIPPED`    | Another node of the job failed; the agent skipped or stopped this one. |

## Logs and secret masking

The agent sends each line of stdout and stderr as one log entry, without its
line terminator. It decodes output as UTF-8 and replaces invalid bytes with
U+FFFD. It splits a line longer than 64 KiB into several lines. Read the logs
in the job page of the web UI, or with `JobService.ListJobLogs` (stored lines,
oldest first) and `JobService.TailJobLogs` (stored lines, then live lines until
the job ends).

The agent masks secret values before it sends a line:

- Only values that come from a `secret_ref` are masked. A literal `value` and a
  trigger input are not.
- The agent splits the secret value into lines and trims each line. Each line
  of at least 4 characters is replaced with `***` wherever it occurs in the
  output, the longest first.
- A line of the secret shorter than 4 characters is not masked.
- Only the exact text is masked. A transformed value, for example base64,
  URL-encoded or split across two output lines, shows as is.

Masking reduces accidental leaks. It is not a security boundary: a step that
reads a secret can always print it in another form.

## Limits

| Limit                                         | Value                    |
| --------------------------------------------- | ------------------------ |
| Nodes per pipeline                            | 1 to 256                 |
| `node_id`                                     | 128 characters           |
| Env entries per node                          | 128                      |
| Env key / env value                           | 255 bytes / 65536 bytes  |
| `working_dir`                                 | 4096 bytes               |
| `exec.command`                                | 4096 bytes               |
| `exec.args`                                   | 256 entries of 65536 bytes |
| `script.script`                               | 1 MiB                    |
| Resolved job (nodes with secrets and inputs)  | 8 MiB, else `RunPipeline` fails with `INVALID_ARGUMENT` |
| Log line                                      | 64 KiB, longer lines are split |

The operating system of the agent also limits the environment and the
arguments of one process (about 1 MiB on macOS). A node over that limit fails
when it starts. A script step is not bound by the argument limit, because the
agent runs it from a file.

## Example: `CreatePipeline`

The request below creates a four-node pipeline. `checkout` clones a
repository with a token from the project secret `github-token`. `test` and
`lint` run in parallel after `build`. `package` waits for both. It uses the
proto JSON form, as `grpcurl` takes it.

```json
{
  "project_id": { "value": "01k7example0project0id0000" },
  "name": "build-and-test",
  "nodes": [
    {
      "node_id": { "value": "checkout" },
      "env": [
        { "key": "GH_TOKEN", "secret_ref": "github-token" },
        { "key": "REPO", "value": "github.com/acme/app.git" }
      ],
      "script": {
        "shell": "SHELL_BASH",
        "script": "git clone \"https://x-access-token:${GH_TOKEN}@${REPO}\" src\ngit -C src checkout \"${GIT_COMMIT:-HEAD}\""
      }
    },
    {
      "node_id": { "value": "build" },
      "deps": [{ "value": "checkout" }],
      "working_dir": "src",
      "exec": { "command": "make", "args": ["build"] }
    },
    {
      "node_id": { "value": "test" },
      "deps": [{ "value": "build" }],
      "working_dir": "src",
      "script": {
        "shell": "SHELL_BASH",
        "script": "make test 2>&1 | tee test.log"
      }
    },
    {
      "node_id": { "value": "lint" },
      "deps": [{ "value": "build" }],
      "working_dir": "src",
      "exec": { "command": "make", "args": ["lint"] }
    },
    {
      "node_id": { "value": "package" },
      "deps": [{ "value": "test" }, { "value": "lint" }],
      "working_dir": "src",
      "env": [{ "key": "OUT_DIR", "value": "dist" }],
      "script": { "script": "mkdir -p \"$OUT_DIR\"\ntar -czf \"$OUT_DIR/app.tar.gz\" build" }
    }
  ]
}
```

`GIT_COMMIT` is not set by Scylla. It comes from a webhook trigger input in
[Triggers](./triggers.md#github); without it the script checks out `HEAD`. The
agent must have `git` and `make` on its `PATH`.

To send it with `grpcurl` (the server serves gRPC reflection), first get a
session token with `AuthService.Login`:

```sh
TOKEN=$(grpcurl -plaintext -d '{"identifier":"admin","password":"admin123"}' \
  localhost:8080 scylla.auth.v1.AuthService/Login | jq -r .token)

grpcurl -plaintext -H "authorization: Bearer $TOKEN" -d @ \
  localhost:8080 scylla.pipeline.v1.PipelineService/CreatePipeline < pipeline.json
```

A session token expires 24 hours after login.

## The web editor format

The pipeline editor of the web UI has a text mode. It uses its own JSON
document, which the UI converts to the request above:

```json
{
  "name": "my-pipeline",
  "projectId": "01k7example0project0id0000",
  "nodes": [
    {
      "id": "checkout",
      "deps": [],
      "kind": "script",
      "shell": "bash",
      "script": "git clone \"https://x-access-token:${GH_TOKEN}@github.com/acme/app.git\" src",
      "workingDir": "",
      "env": [{ "key": "GH_TOKEN", "kind": "secret", "secretRef": "github-token" }]
    },
    {
      "id": "build",
      "deps": ["checkout"],
      "kind": "exec",
      "command": "make",
      "args": ["build"],
      "workingDir": "src",
      "env": [{ "key": "MODE", "kind": "literal", "value": "release" }]
    }
  ]
}
```

| Editor key       | Pipeline field                                   |
| ---------------- | ------------------------------------------------ |
| `id`             | `node_id`                                        |
| `deps`           | `deps`, as plain strings                         |
| `kind`           | `"script"` or `"exec"`                           |
| `shell`          | `"sh"` or `"bash"`; any other value is `"sh"`    |
| `script`         | `script.script`                                  |
| `command`, `args` | `exec.command`, `exec.args`                     |
| `workingDir`     | `working_dir`                                    |
| `env[].kind`     | `"literal"` with `value`, or `"secret"` with `secretRef` |

The same rules as above apply to every field.
