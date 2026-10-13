# Running an agent

An agent is a `scylla-agent` process. It runs the jobs of the control plane on
its own machine. This page covers its identity, its connection, its workspace
and its life cycle. The flags are listed in
[Configuration](./configuration.md#agent-scylla-agent).

## Identity and access

An agent authenticates as an *App*, a machine account owned by one
organization.

- **Agents** > **New Agent** in the web UI (`AgentAdminService.CreateAgent`)
  creates the App, its first secret and a grant of the Organization Agent role
  on the organization. The agent may then run the jobs of every project of the
  organization.
- The Project Agent role limits an App to the jobs of one project. See the
  [Access model](./access-model.md#agents).

The control plane gives a job to an agent only when the agent holds
`executeJob` on the pipeline of the job. Both agent roles grant it. An App
acts only in the organization that owns it.

## Secrets and tokens

The secret is 64 lowercase hex characters. The server returns it once, at
creation, and stores only a hash of it. An App may hold several secrets, each
with a label. If you lose a secret, add a new one to the same App
(`AppService.CreateAppSecret`); you do not need a new agent.

At each connection the agent exchanges its App id and secret for a new app
token (`AppAuthService.IssueToken`). A token expires **30 days** after it is
issued. The control plane checks the token once, when the stream opens. The
agent asks for a fresh token at every reconnection, so you never renew a token
by hand. The secret itself does not expire.

To rotate a secret:

1. Add a new secret to the App.
2. Restart the agent with the new secret.
3. Revoke the old secret (`AppService.RevokeAppSecret`). This closes the
   stream once; the agent reconnects with the new secret.

These actions close the stream of the agent at once:

- disabling the App (`SetAppActive`) or deleting it;
- disabling or revoking a secret of the App;
- removing the grant that lets the App run jobs.

When the control plane refuses the credentials, the agent logs the reason and
exits; it does not retry.

## Connection

The agent connects to `--control-plane-url` on the same port as the UI and
the API. It makes only outbound connections. An `https://` URL uses TLS with
the root certificates of the system.

Once the stream is open, the agent sends a hello with its version, operating
system, architecture, host name, CPU count and memory.
`AgentAdminService.GetAgent` returns them as `host`. The control plane does
not use them to place jobs.

The control plane pings the connection every 20 seconds and closes it when a
ping gets no answer within 10 seconds. The agent does the same on its side.

### One App, one process

Run one agent process per App. A second process with the same App replaces
the first: the older stream ends with `ALREADY_EXISTS`, and the older process
exits. To run several agents, create one App for each.

### Reconnection

When the stream closes or a connection fails, the agent waits and tries
again. The delay starts at `--reconnect-backoff-secs` (3 seconds by default)
and doubles after each failure, up to 60 seconds. A connection that stays up
for 5 seconds or more resets the count.

After `--max-reconnect-attempts` failures in a row (10 by default) the agent
exits. With `0` it retries forever. Run it under a supervisor, such as
systemd or `docker run --restart`, if it must stay up.

The agent exits at once, without a retry, when the control plane answers
`UNAUTHENTICATED`, `PERMISSION_DENIED`, `NOT_FOUND` or `ALREADY_EXISTS`.

A job keeps running while the agent reconnects. Its status and log messages
wait in a buffer (`--publish-buffer-size`) and go out on the next stream. The
hello of the new stream lists the jobs that the agent still runs:

- a running job of this agent that is not in the list becomes `ORPHANED`;
- a listed job that the control plane does not record as running on this
  agent gets a cancel order.

### Orphaned jobs

A running job whose agent stays disconnected for **60 seconds** after its
last contact becomes `ORPHANED`. The control plane checks every 30 seconds,
so the change can take up to about 90 seconds. A job placed on an agent but
not started goes back to the pending pool when the stream closes.

## Jobs

An agent runs **one job at a time**. The control plane gives the oldest
pending job to a connected agent that may run it and has no unfinished job.
To run jobs in parallel, run more agents.

Within a job, nodes whose dependencies are met run at the same time. See
[Pipelines](./pipelines.md) for the step rules, the environment of a step and
the failure rules.

## Workspace

At start, the agent creates `--workspace-root` if needed and checks that it
can write there. If not, it exits with an error.

For each job the agent creates `<workspace root>/<job_id>`. All nodes of the
job share this directory; `SCYLLA_WORKSPACE` holds its absolute path. Script
steps are written under `.scylla/` in it. When the job ends, the agent removes
the directory. With `--keep-workspace` it keeps it, to inspect the files
after a failure; remove old workspaces yourself in that case.

The default root, `/var/lib/scylla/workspaces`, is usually not writable by a
normal user. Outside the container image, pass a directory that the agent's
user owns, for example `--workspace-root "$HOME/.scylla/workspaces"`.

## Run with Docker

```sh
docker run -d --restart unless-stopped \
  -e SCYLLA_CONTROL_PLANE_URL=https://ci.example.com \
  -e SCYLLA_APP_ID=<APP_ID> \
  -e SCYLLA_APP_SECRET=<APP_SECRET> \
  -v scylla-workspaces:/var/lib/scylla/workspaces \
  godlyjaaaaj/scylla-agent:latest
```

The image runs as the user `appuser` (uid 10001) and creates
`/var/lib/scylla/workspaces` with that owner. A named volume mounted there
keeps that owner.

The steps run inside the container, with its tools. The image is Debian
bookworm slim with `curl` and CA certificates added; it has no `git`, `make`
or compiler. Build an image on top of it with the tools of your pipelines, or
run the agent directly on a host that has them.

## Run as a service

On a host, build the binary from a checkout with
`cargo build --release -p scylla-agent` and run it under a supervisor. A
minimal systemd unit:

```ini
[Unit]
Description=Scylla agent
After=network-online.target
Wants=network-online.target

[Service]
User=scylla
Environment=SCYLLA_CONTROL_PLANE_URL=https://ci.example.com
Environment=SCYLLA_WORKSPACE_ROOT=/var/lib/scylla/workspaces
EnvironmentFile=/etc/scylla/agent.env
ExecStart=/usr/local/bin/scylla-agent
Restart=always

[Install]
WantedBy=multi-user.target
```

`/etc/scylla/agent.env` holds `SCYLLA_APP_ID` and `SCYLLA_APP_SECRET`, readable
by root only. The user `scylla` must own the workspace root.

## Shutdown

On `SIGTERM` or Ctrl-C the agent stops its running jobs: it sends `SIGTERM` to
their processes, then `SIGKILL` after 5 seconds. It reports each of these
jobs as `FAILED`, then closes the stream. It exits within about 10 seconds.

## Troubleshooting

| Symptom                                        | Check                                                         |
| ---------------------------------------------- | ------------------------------------------------------------- |
| The agent exits with "refused the app credentials" | The App id, the secret, and that the App and the secret are enabled. |
| The agent never shows as connected             | The agent can reach the URL; from a container, `localhost` is the container. |
| Jobs stay pending                              | The agent is connected, belongs to the organization of the project, holds an agent role, and is not busy with another job. |
| A job ends as `ORPHANED`                       | The agent lost its connection or restarted during the job.    |
| A node fails at once with a spawn error        | The command or the shell is not on the agent's `PATH`. The error is in the stderr log of the node. |
