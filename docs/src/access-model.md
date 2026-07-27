# Access model

How Scylla decides what someone may do. This page is the reference the code is
written against: if an implementation disagrees with it, the implementation is
wrong.

## Principles

**Every right comes from an explicit access.** Belonging to something grants no
capability. If you can do X, someone gave you a role that contains X.

**Organization membership is a prerequisite, not a right.** It makes the
organization visible and nothing else. It gates every access beneath it: leaving
the organization extinguishes them all, at once, with no cleanup to perform.

**An access covers its scope and everything inside it.** A role on an
organization covers its projects. A role on a project covers only that project.

**Rights add up and are never subtracted.** There are no deny rules. Two
accesses give the union of the two. To know what someone can do, list their
accesses; nothing elsewhere can silently cancel one.

## Objects

| Object                  | Definition                                                                     |
| ----------------------- | ------------------------------------------------------------------------------ |
| Organization            | The customer. Billing and ownership boundary.                                   |
| Organization membership | A person attached to the organization. Sees the organization and its member list. |
| Project                 | A unit of work inside an organization. Holds pipelines, jobs and secrets.       |
| Role                    | A named permission set, valid at one scope kind. Editable by the tenant.        |
| Grant                   | "This principal holds this role on this scope." The only source of authority.   |

**Visibility rule.** You see a thing if you hold a role on it, or on something
that contains it. There is no other way to see anything.

## Scopes

| Scope        | A role granted here covers          | Typical holder            |
| ------------ | ----------------------------------- | ------------------------- |
| System       | Every organization                  | Platform operators        |
| Organization | The organization and all its projects | Customer administrators |
| Project      | That project only                   | Delivery teams            |

## Being on a project

Being on a project means **holding a role on it**. It is one fact, not two: there
is no membership row separate from the grant, so the two can never disagree.

- The people on a project are exactly the holders of a grant scoped to it.
- Adding someone is granting them a role.
- Removing someone is revoking it.
- There is no state between "no access" and "some role".

Organizations are deliberately different. There, membership without a role is a
real state: you belong to the company but have not been assigned yet. An
organization is an attachment; a project is an assignment. The first can be
empty of content, the second cannot.

## Builtin roles

Shipped so a customer is usable without configuring anything. Custom roles can be
created at any scope with any permission set.

| Role                     | Scope        | Confers                                                        |
| ------------------------ | ------------ | -------------------------------------------------------------- |
| System Admin             | System       | Everything, everywhere                                          |
| Organization Admin       | Organization | Everything in the organization, including managing its accesses |
| Organization Viewer      | Organization | Read every project and run in the organization                  |
| Project Admin            | Project      | Everything on the project, including managing its accesses      |
| Project Developer        | Project      | Create, edit, run pipelines; read jobs and logs; list secrets   |
| Project Viewer           | Project      | Read the project, its pipelines, its jobs and logs              |
| Organization Agent       | Organization | Machine app: pull and run the organization's jobs               |
| Project Agent            | Project      | Machine app: pull and run the project's jobs                    |
| Organization Trigger Runner | Organization | Machine app: fire the organization's pipelines               |

## Guarantees

**An organization always keeps at least one human administrator.** The last one
can be neither removed nor demoted; the error says to appoint another first.

**A project may end up with no administrator.** Organization administrators cover
it and can reopen access, so this is recoverable rather than a dead end.

**Nobody can grant more than they hold.** A project administrator cannot award
themselves an organization role.

**Removing someone from an organization is the kill switch.** Immediate, total,
no manual cleanup.

## Deliberate non-goals

Written down so they do not creep back in.

- **No per-pipeline or per-job rights.** The project is the finest grain. Two
  confidentiality levels in one project means two projects.
- **No deny rules.** Additive only.
- **No time-limited access.**
- **No access requests with approval.**
- **No groups or teams.** Access is granted person by person. This is the known
  ceiling: past a few dozen projects and people it becomes tedious, and groups
  are the natural extension when that day comes.
