# Users, orgs & access

The practical view of who can do what; the engine enforcing it is
[the authorization model](../architecture/authorization.md).

## Signing in

People sign in as **Users**, two ways:

- **Username + password.** The first boot creates the bootstrap `admin` user.
- **GitHub OAuth**, with a linked GitHub identity.

A successful login opens a **session** — an opaque token attached to each
request and rejected once expired.

## Invitations & membership

You join an organization or project through **membership**: an existing member
with the right to manage access **invites** you; accepting creates the
membership. Membership is deliberately plain — it records only *that* you
belong, with no role stored on the row. Being a member grants baseline
read/operate access to that tenant's resources; anything more comes from a
**grant**.

## Scopes, roles & grants

**Scope** — the level something applies to, nested broadest-to-narrowest:
`System ⊃ Organization ⊃ Project`. A grant at one scope reaches everything
beneath it — a grant on an organization also covers its projects.

**Role** — a named bundle of permissions. The builtins:

| Role | Binds to | Confers |
|------|----------|---------|
| `system-admin` | System | Full control over everything (System is the root). |
| `organization-admin` | Organization | Full control of an org and everything beneath it. |
| `project-admin` | Project | Full control of a project and everything beneath it. |
| `organization-agent` | Organization | Machine app: read pipelines, execute jobs, write status/logs. |
| `project-agent` | Project | The same, within one project. |

Admins can also define **custom roles** (a chosen permission set) owned by an
organization.

**Grant** — the one mechanism tying it together: *"this principal holds this
role (or single permission) within this scope."* A **principal** is a User or
an App. "Alice is org-admin of Org A" and "this agent App may run pipelines in
Org A" are both just grants.

Creating an org or project automatically gives you an owner grant on it
(`organization-admin` / `project-admin` by default — an admin can rebind which
role creators receive).

## Guard-rails

Admins assign and revoke grants from the permission views. Scylla enforces:

- **No privilege escalation.** You can only grant capabilities you already
  hold at that scope — a narrow "manage grants" holder can't grant themselves
  full admin.
- **A scope keeps its last human owner.** Revoking the final `*-admin` *user*
  of an org or project is blocked — add another owner first.
- **Revoking an agent's grant disconnects it.** Its live stream drops
  immediately, so a no-longer-authorized worker stops at once.
- **Changes are live** — effective on the next authorization check, no
  restart.

For the permission catalog, Cedar policy generation, and evaluation, see
[the authorization model](../architecture/authorization.md).
