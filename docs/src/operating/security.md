# Security

The shipped defaults are tuned for a friction-free first run on localhost.
Work through this list before exposing Scylla to anyone else.

## Change the bootstrap credentials

The first boot creates an `admin` user from `[bootstrap]`, defaulting to
`admin` / `admin123` — **public knowledge**. Either set a strong
`[bootstrap].password` (and ideally a real `username` / `email`) before the
*first* boot (the account is created then), or sign in afterwards and rotate
the password / replace the account.

## Secret encryption master key

Project secrets are encrypted at rest under `[secrets].master_key` — 64 hex
chars (32 bytes). Two rules:

1. **Replace the sample key.** The dev configs ship a well-known key. Generate
   a fresh one and **inject it at deploy time** (environment / secret
   manager), never in a committed file.
2. **Guard it like the secrets it protects.** Anyone with the master key can
   decrypt every stored secret.

If `[secrets]` is absent the secret store is disabled and secret operations
fail with a clear error — a missing key never silently stores plaintext.

> Rotating the master key re-keys the cipher, not the already-encrypted rows.
> Treat a key change as a migration: re-enter (delete + recreate) existing
> secrets under the new key.

## CORS

The browser talks to the API directly, so the control plane sets CORS.
`[cors].allow_origins` defaults to `*` for local dev — **pin it to your UI's
origin** in production (as `prod.toml` shows). An over-broad origin list lets
any site drive a signed-in user's API.

## Webhook signatures

Inbound webhooks are authenticated by an HMAC-SHA256 signature of the raw
body. Keep each trigger's signing secret safe — it is shown once at creation
and stored encrypted. The endpoint returns an opaque `404` for unknown or
disabled triggers so it never reveals which ids exist. See
[Triggers](../using/triggers.md).

## Transport (TLS)

The control plane speaks plaintext gRPC / gRPC-Web on `50051` and plain HTTP
on the webhook port. Terminate TLS in front of it (reverse proxy / load
balancer) for any non-local deployment, and make sure the browser's
`VITE_API_URL` and your webhook `public_base_url` use `https://`.

## Principle of least privilege

Give agents the narrow `organization-agent` / `project-agent` role — never an
admin grant. The grant model also blocks privilege escalation and protects a
scope's last human owner; see [Users, orgs & access](../using/access.md).
