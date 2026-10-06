# Access model

How Scylla decides what someone may do. This page is the contract the code is
written against: if an implementation disagrees with it, the implementation is
wrong.

## The rule

> You may do X on Y if you hold a role granting X on Y, or on something that
> contains Y.

That is the whole model. There is no second mechanism, no implicit tier, no
membership that quietly confers rights.

## Objects

| Object       | Definition                                                                      |
| ------------ | ------------------------------------------------------------------------------- |
| Organization | The customer. Billing and ownership boundary.                                    |
| Project      | A unit of work inside an organization. Holds pipelines, jobs and secrets.        |
| Role         | A named permission set, valid at one scope kind. Tenants can define their own.   |
| Grant        | "This principal holds this role on this scope." The only source of authority.    |

There is deliberately **no membership table**. Belonging somewhere means holding
a grant on it or on something that contains it, so `grants` is the single
relation between a principal and the tenancy tree. One list, which can never
disagree with itself about who is where.

## Scopes

| Scope        | A role granted here covers            | Typical holder            |
| ------------ | ------------------------------------- | ------------------------- |
| System       | Every organization                    | Platform operators        |
| Organization | The organization and all its projects | Customer administrators   |
| Project      | That project only                     | Delivery teams            |

## Items inside a project

Pipelines, jobs, triggers and secrets are not scopes. You cannot grant a role
on them. A check on one of these items finds the project that holds the item,
and then applies the rule above to that project.

A check on one trigger uses the permissions of its pipeline: `manageTriggers`
to read, change, enable, disable or delete it, and `runPipeline` to fire it
now. A change also needs `runPipeline`. Thus the roles that give these
permissions on the pipeline also give them on its triggers.

A run reads the value of each secret that its pipeline refers to. Thus a
principal that may create, update or run a pipeline of a project may read the
secrets of that project through a run. `listSecrets` shows the names only; it
is not the boundary. Put a secret that some of these principals must not read
in a different project.

To cancel a job, a caller needs `updateJob` on the job. The Project Developer
role gives it, so a principal that may run the pipelines of a project may also
cancel their jobs.

## Agents

An agent is an app. The control plane gives a job to an agent only when the
agent holds `executeJob` on the pipeline of the job, through a grant on its
project or on its organization. The dispatcher reads this rule from the grants
of the agent, not from a check, so it writes no audit row. Only the agent that
a job is placed on may report the status and the output of that job, and only
until the job ends. When an app loses its grant, is disabled or is deleted,
the control plane closes its stream.

## Unknown ids

A check on an item finds its nearest known container: the pipeline of a job
or a trigger, the project of a pipeline or a secret, the organization of a
project, an app or an invitation. A check on an id that does not exist finds
no container, so the item is under System. Only a System grant reaches it.
Thus a caller without a System grant gets "forbidden" and does not learn if
the item exists. A caller with a System grant gets "not found".

A write that refers to a container or a principal that does not exist, for
example a project in an unknown organization or a grant to an unknown user,
fails with "failed precondition".

## Invitations

An invitation is not a scope. A check on one invitation, for example to revoke
it, finds the organization that holds the invitation. Then it applies the rule
above to that organization, with the permission `manageInvitations`. Thus the
roles that give this permission on the organization also give it on its
invitations.

A check on an unknown invitation finds no organization (see
[Unknown ids](#unknown-ids)).

## App secrets

An app secret is not a scope. A check on one app secret, for example to revoke
it or to disable it, finds the app that holds the secret, and then the
organization that holds the app. Then it applies the rule above to that
organization, with the permission `deleteApp`. Thus the roles that give this
permission on the organization also give it on the secrets of its apps.

A check on an unknown app secret finds no app (see
[Unknown ids](#unknown-ids)).

## Revoking a grant

A check to revoke one grant finds the scope that holds the grant. Then it
applies the rule above to that scope, with the manage-grants permission of the
scope kind: `manageProjectGrants` for a project grant, `manageOrgGrants` for an
organization grant and `manageSystemGrants` for a System grant. Thus the roles
that give this permission on the scope also give it on the grants of the scope.

A check on an unknown grant finds no scope, and uses the System rule. Only a
grant that gives `manageSystemGrants` reaches it. A caller without it gets
"forbidden". A caller with it gets a success, and nothing changes.

## Being somewhere

Being in an organization, or on a project, means **holding a role on it**. It is
one fact, not two.

- The people on a project are exactly the holders of a grant scoped to it.
- Adding someone is granting them a role; removing them is revoking it.
- There is no state between "no access" and "some role". Someone who should
  belong without being able to act holds the `organization-member` role, which
  confers only the ability to see that the organization exists.

Holders of an organization-wide grant do not appear in a project's people list.
They administer the organization; the hierarchy already gives them the access
they need.

## Builtin roles

Shipped so a tenant is usable without configuring anything. Custom roles are
described under "Custom roles".

| Role                        | Scope        | Confers                                                        |
| --------------------------- | ------------ | -------------------------------------------------------------- |
| System Admin                | System       | Everything, everywhere                                          |
| Organization Admin          | Organization | Everything in the organization, including managing its accesses |
| Organization Viewer         | Organization | Read every project and run in the organization                  |
| Organization Member         | Organization | Sees the organization exists. Nothing else                      |
| Project Admin               | Project      | Everything on the project, including managing its accesses      |
| Project Developer           | Project      | Create, edit, run pipelines; read and cancel jobs; read logs; list secrets; read the secrets through a run |
| Project Viewer              | Project      | Read the project, its pipelines, its jobs and logs              |
| Organization Agent          | Organization | Machine app: pull and run the organization's jobs               |
| Project Agent               | Project      | Machine app: pull and run the project's jobs                    |
| Organization Trigger Runner | Organization | Machine app: fire the organization's pipelines                  |

## Custom roles

A role with no owner is a platform role: the builtin roles and the custom roles
of the system administrators. Only a holder of `manageRoles` (System) creates,
edits or deletes one. Every organization sees the platform roles.

A role with an owner belongs to that organization. A holder of `manageOrgRoles`
on the organization creates, edits and deletes it; `organization-admin` has this
permission through full control. Such a role:

- is organization or project scoped, never system scoped;
- is seen, listed and granted only in its organization. For another
  organization, it is the same as a role that does not exist;
- does not take the name of a platform role. Two organizations may each have a
  role with the same name. Names are compared without regard to case. A system
  administrator may still create a platform role with the name of an
  organization role: that organization then sees both, each in its group.

A change to one role is checked on the role: the access model applies
`manageOrgRoles` for a role that an organization owns, and `manageRoles` for a
platform role. So an organization administrator cannot edit a builtin role, or
a role of another organization.

A role holds only permissions that its author holds at that organization, or at
the system for a platform role. A role is deleted only when no grant and no
pending invitation names it. The builtin roles are never deleted.

## Guarantees

**A role of an organization is granted only inside that organization.** The use
case refuses it elsewhere, and a database trigger refuses such a grant or
invitation too. When the organization is deleted, its roles and their grants go
with it.

**The system and each organization always keep at least one human
administrator** (`system-admin`, `organization-admin`). You cannot revoke the
last one, remove all its access or delete its user. The error tells you to
appoint another administrator first. An app does not count as an
administrator.

**A project may end up with no administrator.** Organization administrators
cover it and can reopen access, so this is recoverable rather than a dead end.

**Nobody can grant more than they hold.** A role is given only by a principal
that holds each of its permissions on the scope, on the organization of the
scope or on System. Full control there permits all roles. An invitation obeys
the same rule, and an invitation without a role counts as
`organization-member`. A project administrator cannot award themselves an
organization role.

**An agent role is for an app only.** A user cannot hold
`organization-agent`, `project-agent` or `organization-trigger-runner`.

**An app acts only in the organization that owns it.** A grant to an app on
another organization, or on a project of another organization, is refused.

**A grant refers to a principal and a scope that exist.** The database refuses
a grant for an unknown user, app, organization or project. When the user, the
app or the scope is deleted, its grants go with it.

**A project-scope grant only goes to someone already in the organization.** A
project administrator distributes access among people the organization has
already accepted; they cannot pull in an arbitrary account from another tenant.
Bringing someone into the organization requires `manageOrgGrants`, which only
organization and system administrators hold.

**Removing someone from an organization strips every access beneath it** in one
operation, projects included (`RevokeAllAccess`). System-scoped grants are never
touched by it, so an organization administrator cannot strip a platform
operator. When `RevokeGrant` removes the last grant of a user on an
organization, it also removes the grants of that user on the projects of the
organization.

## Revocation timing

The control plane compiles the grants and the roles into a policy set that it
keeps in memory. The table `authz_version` holds one number. A database trigger
increases it in the same transaction as each change to `grants`, `roles` or
`role_permissions`. This includes the grants that a delete of a user, an app, an
organization or a project removes by cascade.

Each check reads the version first. If the version is not the one of the set in
memory, the control plane builds the set again before it decides. Thus a grant
or a revocation applies on the next check, in every control plane that uses the
database. No code path has to ask for a rebuild.

If the rebuild fails, the check fails. The control plane does not decide with a
set that can still hold a revoked grant.

## Deliberate non-goals

Written down so they do not creep back in.

- **No per-pipeline or per-job rights.** The project is the finest grain. Two
  confidentiality levels in one project means two projects.
- **No deny rules.** Additive only, so working out what someone can do never
  requires hunting for something that cancels it.
- **No time-limited access.**
- **No access requests with approval.**
- **No groups or teams.** Access is granted person by person. This is the known
  ceiling: past a few dozen projects and people it becomes tedious, and groups
  are the natural extension when that day comes.
