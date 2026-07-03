# Secrets

> 🚧 *Chapter in progress.*

Project-scoped secrets injected into pipeline execution. Values are write-only — set once, never returned.

## Creating a secret

<!-- Project scope, name + value; value is write-only. -->

## How secrets reach a job

<!-- Encrypted at rest (XChaCha20-Poly1305), decrypted at dispatch. -->

## Rotation & deletion

<!-- Overwrite value; delete metadata. -->
