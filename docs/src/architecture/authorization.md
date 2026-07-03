# Authorization model

> 🚧 *Chapter in progress.*

Scylla's authorization is Cedar-backed RBAC with an ABAC base. This chapter is the deep dive; the vocabulary is fixed in the [Glossary](../reference/glossary.md).

## The seven words

<!-- Permission · Role · Scope · Grant · Principal · Caller · Policy (+ Resource). -->

## Grants: the one mechanism

<!-- (principal, role, scope) triple; System scope = tenancy root. -->

## Roles & permissions

<!-- Closed permission catalog; dynamic roles; builtin roles seeded on boot. -->

## Cedar generation

<!-- Policy set generated from roles+grants; static ABAC base; admin cedar_policies. -->

## PermissionService & enforcement

<!-- check(caller, Permission) fail-closed; audit log row per check. -->
