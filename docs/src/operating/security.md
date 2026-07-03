# Security

> 🚧 *Chapter in progress.*

What to lock down before exposing Scylla beyond localhost.

## Change the bootstrap credentials

<!-- Default admin/admin123 — rotate immediately. -->

## Secret encryption master key

<!-- [secrets].master_key — replace the dev key, inject at deploy, rotation notes. -->

## CORS

<!-- allow_origins in prod.toml — pin to your domain. -->

## Webhook signatures

<!-- HMAC-SHA256 verification; keep trigger secrets safe. -->

## Transport

<!-- gRPC/gRPC-Web; TLS termination in front. -->
