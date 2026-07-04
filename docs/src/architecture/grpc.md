# gRPC protocol

All API traffic is gRPC. The browser speaks **gRPC-Web**; agents and internal
calls speak native gRPC. Both share one port (`50051`) — there is no separate
broker or REST API (the only HTTP endpoint is the optional webhook ingress on
`8088`).

## The protocol crate

`scylla-protocol` owns the `.proto` definitions under `proto/` and their generated
bindings — **Rust** (via `prost` / `tonic`, in `build.rs`) and **TypeScript** (via
`protobuf-ts`). Backend and frontend both import from here, so the wire contract
has exactly one source.

## Services

The proto surface, one file per area:

| Proto | Service | Covers |
|-------|---------|--------|
| `auth.proto` | AuthService | Login, sessions. |
| `user.proto` | UserService | User accounts. |
| `organization.proto` | OrganizationService | Orgs + membership. |
| `project.proto` | ProjectService | Projects + membership. |
| `pipeline.proto` | PipelineService | Pipeline CRUD + `RunPipeline`. |
| `job.proto` | JobService | Jobs, node state, logs. |
| `trigger.proto` | TriggerService | Cron / webhook triggers. |
| `secret.proto` | SecretService | Project secrets. |
| `app.proto` | AppService / AppAuthService | Machine principals + token exchange. |
| `agent.proto` | AgentService | The worker stream (`Open`). |
| `agent_admin.proto` | AgentAdminService | Agent administration. |
| `permission.proto` | Permissions / roles / grants | The authz surface. |
| `invitation.proto` | InvitationService | Org/project invitations. |
| `oauth.proto` | OAuthService | GitHub OAuth login. |
| `registration.proto` | RegistrationService | Sign-up. |
| `common.proto` | — | Shared ids, pagination, `Shell`, step messages. |

## The worker stream

`AgentService.Open` is a **bidirectional stream**: the control plane sends
`AgentDown` messages (chiefly `JobDispatch`) and the agent sends `AgentUp` messages
(node status events and log lines). One open stream is the agent's presence; there
is no polling and no broker. The mechanics are in
[Pipeline execution](./execution.md).

## gRPC-Web for the browser

The frontend can't speak raw HTTP/2 gRPC, so the control plane wraps the services
with **`tonic-web`**, and the UI uses `@protobuf-ts/grpcweb-transport`. This is why
CORS matters: the browser makes the calls directly, so `[cors]` must allow the UI's
origin and expose the `grpc-status*` headers (see
[Configuration](../operating/configuration.md)).

## Authentication

Every call carries `authorization: Bearer <token>`. An async **auth interceptor**
resolves it to a caller:

1. Try it as a **user session** token (`SessionRepository`).
2. Failing that, try it as an **app token** (`AppTokenRepository`).
3. Unknown or expired → reject with `Unauthenticated`.

On success it attaches an `AuthContext { caller }` (`CallerContext::User` or
`::App`) to the request extensions, which the handlers thread into the use cases
for [authorization](./authorization.md). Apps obtain their bearer token by
exchanging their id + secret via `AppAuthService.IssueToken`.
