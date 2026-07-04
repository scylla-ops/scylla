# Secrets

Secrets are project-scoped, encrypted values you reference from a pipeline
without ever putting them in the pipeline definition. A secret's **value is
write-only** — set once at creation, never returned again; only its metadata
(name, description, timestamps) can be read back.

## Lifecycle

On creation the value is encrypted at rest with authenticated encryption
(XChaCha20-Poly1305) under the deployment's master key. From then on:

- **List** returns metadata only — never the value.
- **Delete** removes the secret.
- There is no read-value and no in-place edit. **Rotation is
  delete-and-recreate**: same name, new value; pipelines referencing the name
  pick up the new value on their next run. Deleting without recreating leaves
  references unresolvable — fix them or recreate the secret.

Every operation is authorization-checked (`CreateSecret`, `ListSecrets`,
`DeleteSecret` on the owning project).

## Using a secret in a pipeline

A node references a secret through its environment: instead of an inline
literal, an env var's source is a **secret reference** — the name of a secret
in the same project (see
[Writing pipelines](./pipelines.md#environment-variables)). At dispatch the
control plane resolves the reference, decrypts the value, and injects the
plaintext as an environment variable for that node. The **agent never talks to
the secret store**, and any secret-sourced value is **redacted** (`***`) from
the node's log output.

> Operators: the master key that protects every secret is deployment
> configuration — see [Security](../operating/security.md).
