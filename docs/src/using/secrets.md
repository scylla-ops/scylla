# Secrets

Secrets are project-scoped, encrypted values you reference from a pipeline
without ever putting them in the pipeline definition. A secret's **value is
write-only** — you set it once at creation and Scylla never returns it again;
only its metadata (name, description, timestamps) can be read back.

## Creating a secret

Create a secret inside a project, giving it a name and its value. On creation the
value is encrypted at rest with authenticated encryption (XChaCha20-Poly1305)
under the deployment's master key, then stored. From then on:

- **List** shows secrets' metadata only — never the value.
- **Delete** removes a secret.
- There is no "read value" and no in-place edit. To change a value, delete the
  secret and create it again.

Every operation is authorization-checked (`CreateSecret`, `ListSecrets`,
`DeleteSecret` on the owning project), so managing secrets requires the right
grant on that project.

## Using a secret in a pipeline

A pipeline node references a secret by name through its environment. Instead of an
inline literal, an env var's source is a **secret reference** — the name of a
secret in the same project. See [Writing pipelines](./pipelines.md#environment-variables).

At dispatch time the control plane resolves the reference: it decrypts the secret
and injects the plaintext as an environment variable for that node. The
**agent never talks to the secret store** — it only ever receives already-resolved
values. And any value that came from a secret is **redacted** (`***`) from the
node's log output, so secrets don't leak into logs.

```
node env: DB_PASSWORD ──secret ref──► "db-password"
                                          │  (resolved + decrypted
                                          │   control-plane-side at dispatch)
                                          ▼
   agent receives DB_PASSWORD=<plaintext>, runs the node,
   and masks the value out of every log line
```

## Rotation & deletion

Because the value is write-only, **rotation is delete-and-recreate**: remove the
old secret and create a new one with the same name and the new value. Pipelines
that reference it by name pick up the new value on their next run.

Deleting a secret leaves any pipeline that references it referencing a name that
no longer resolves — fix those references or recreate the secret.

> Operators: the master key that protects every secret is deployment
> configuration. Rotating or protecting it is covered in
> [Security](../operating/security.md).
