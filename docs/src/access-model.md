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

A check on an unknown secret or trigger finds no project. Only a System grant
reaches it. Thus a caller without a System grant gets "forbidden" for an
unknown secret or trigger, and does not learn if it exists. A caller with a
System grant gets "not found".

## Invitations

An invitation is not a scope. A check on one invitation, for example to revoke
it, finds the organization that holds the invitation. Then it applies the rule
above to that organization, with the permission `manageInvitations`. Thus the
roles that give this permission on the organization also give it on its
invitations.

A check on an unknown invitation finds no organization. As for an unknown
secret or trigger, only a System grant reaches it. A caller without a System
grant gets "forbidden", and a caller with a System grant gets "not found".

## App secrets

An app secret is not a scope. A check on one app secret, for example to revoke
it or to disable it, finds the app that holds the secret, and then the
organization that holds the app. Then it applies the rule above to that
organization, with the permission `deleteApp`. Thus the roles that give this
permission on the organization also give it on the secrets of its apps.

A check on an unknown app secret finds no app. As for an unknown invitation,
only a System grant reaches it. A caller without a System grant gets
"forbidden", and a caller with a System grant gets "not found".

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

Shipped so a tenant is usable without configuring anything. Custom roles can be
created at any scope with any permission set.

| Role                        | Scope        | Confers                                                        |
| --------------------------- | ------------ | -------------------------------------------------------------- |
| System Admin                | System       | Everything, everywhere                                          |
| Organization Admin          | Organization | Everything in the organization, including managing its accesses |
| Organization Viewer         | Organization | Read every project and run in the organization                  |
| Organization Member         | Organization | Sees the organization exists. Nothing else                      |
| Project Admin               | Project      | Everything on the project, including managing its accesses      |
| Project Developer           | Project      | Create, edit, run pipelines; read jobs and logs; list secrets   |
| Project Viewer              | Project      | Read the project, its pipelines, its jobs and logs              |
| Organization Agent          | Organization | Machine app: pull and run the organization's jobs               |
| Project Agent               | Project      | Machine app: pull and run the project's jobs                    |
| Organization Trigger Runner | Organization | Machine app: fire the organization's pipelines                  |

Only a system administrator may edit the role catalog (`manageRoles` applies to
the System scope). Tenants pick from it; they do not yet redefine it.

## Guarantees

**An organization always keeps at least one human administrator.** The last one
can be neither removed nor demoted; the error says to appoint another first.

**A project may end up with no administrator.** Organization administrators
cover it and can reopen access, so this is recoverable rather than a dead end.

**Nobody can grant more than they hold.** A project administrator cannot award
themselves an organization role.

**A project-scope grant only goes to someone already in the organization.** A
project administrator distributes access among people the organization has
already accepted; they cannot pull in an arbitrary account from another tenant.
Bringing someone into the organization requires `manageOrgGrants`, which only
organization and system administrators hold.

**Removing someone from an organization strips every access beneath it** in one
operation, projects included (`RevokeAllAccess`). System-scoped grants are never
touched by it, so an organization administrator cannot strip a platform
operator.

## Revocation timing

Grants are compiled into an in-memory policy set and rebuilt on change. A
revocation therefore takes effect when the control plane reloads that set, which
happens synchronously in the process that performed the revocation.

This is exact for the current single-process deployment. **It would not hold
across replicas**: a second control plane would keep serving the old set until
its own next reload. Running more than one replica requires a cross-process
invalidation channel first (a `pg_notify` on `grants` with a listener calling
`reload`). Until that exists, treat single-process as a deployment constraint,
not an implementation detail.

If a reload fails, the previous policy set is deliberately kept so that no check
is ever served by a broken set. The rows are already gone at that point, so the
failure is logged at error level and returned to the caller: the store and the
live set are out of step until the next successful reload.

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
