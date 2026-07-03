# Users, orgs & access

This chapter is the practical view of who can do what in Scylla: signing in,
joining orgs and projects, and how roles decide capabilities. The engine that
enforces it all is described in [the authorization model](../architecture/authorization.md).

## Signing in

People sign in as **Users**. Two ways in:

- **Username + password.** The first boot creates the bootstrap `admin` user.
- **GitHub OAuth.** Sign in with a linked GitHub identity.

A successful login opens a **session** — an opaque token attached to each request
and rejected once expired.

## Invitations & membership

You join an organization or project through **membership**. An existing member
with the right to manage access can **invite** a user; accepting the invitation
creates the membership.

Membership is deliberately plain: it records only *that* a user belongs to an org
or project — no role or permission is stored on the membership row itself. Being a
member grants baseline read/operate access to that tenant's resources; anything
more comes from a **grant** (below).

## Scopes, roles & grants

Three ideas decide what you can do.

**Scope** — the level something applies to, nested broadest-to-narrowest:

```
System  ⊃  Organization  ⊃  Project
```

A capability at one scope reaches everything beneath it — a grant on an
organization also covers its projects.

**Role** — a named bundle of permissions. The builtin roles:

| Role | Binds to | Confers |
|------|----------|---------|
| `system-admin` | System | Full control over everything (System is the root). |
| `organization-admin` | Organization | Full control of an org and everything beneath it. |
| `project-admin` | Project | Full control of a project and everything beneath it. |
| `organization-agent` | Organization | Machine app: pull and run the org's jobs. |
| `project-agent` | Project | Machine app: pull and run a project's jobs. |

Beyond the builtins, admins can define **custom roles** (a chosen set of
permissions) owned by an organization.

**Grant** — the one mechanism that ties it together: *"this principal holds this
role (or single permission) within this scope."* A **principal** is a User or an
App. So "Alice is org-admin of Org A" and "this agent App may run pipelines in
Org A" are both just grants.

When you create an org or project, you automatically get an owner grant on it (the
`organization-admin` / `project-admin` role by default — an admin can rebind which
role new creators receive).

## Managing access in the UI

Admins assign and revoke grants from the permission views. Scylla enforces a few
guard-rails so you can't lock yourself — or a tenant — out:

- **No privilege escalation.** You can only grant capabilities you already hold at
  that scope. A user with a narrow "manage grants" permission can't grant
  themselves full admin.
- **A scope keeps its last human owner.** Revoking the final `*-admin` *user* of an
  org or project is blocked — add another owner first.
- **Revoking an agent's grant disconnects it.** Pull an App's grant and its live
  agent stream drops immediately, so a no-longer-authorized worker stops at once.
- **Changes are live.** Granting, revoking, or editing a role takes effect on the
  next authorization check — no control-plane restart.

For the permission catalog, Cedar policy generation, and how checks are evaluated,
see [the authorization model](../architecture/authorization.md).
