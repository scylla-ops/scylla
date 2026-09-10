# Release image definitions for `docker buildx bake`.
#
# Both published images in one declarative file. Bake builds every target of a
# group concurrently against a single BuildKit session, which is why this
# replaced a loop of `docker buildx build` calls: `control-plane` and `agent`
# share the whole cargo-chef dependency layer, and building them in one bake
# cooks it once instead of twice.
#
# Always pass `-f docker-bake.hcl`: without it bake also loads
# docker-compose.yaml and turns its services into extra, single-arch targets.
#
#   VERSION=0.4.0 docker buildx bake -f docker-bake.hcl --print release
#   VERSION=0.4.0 docker buildx bake -f docker-bake.hcl --push release
#
# `just release <version>` is the wrapper; the process is in RELEASING.md.

variable "VERSION" {
  default     = ""
  description = "Release version, e.g. 0.4.0-beta. Required: an empty value tags nothing."
}

variable "DOCKER_USER" {
  default     = "godlyjaaaaj"
  description = "Registry account the images are published under."
}

variable "PLATFORMS" {
  default     = "linux/amd64,linux/arm64"
  description = "Comma-separated platforms for the multi-arch manifest."
}

variable "GIT_SHA" {
  default     = ""
  description = "Commit the images were built from, recorded as an OCI label."
}

variable "LATEST" {
  default     = "true"
  description = "Also tag `latest`. Set false when publishing something that must not become the default pull — a re-release of an older version, or a build nobody should get by accident."
}

variable "CACHE" {
  default     = ""
  description = "Registry ref prefix for a shared build cache, e.g. godlyjaaaaj/scylla-buildcache. Empty disables it and relies on the local builder cache."
}

# The version tag always; `latest` unless told otherwise.
#
# Scylla is pre-1.0 and every release so far has been a beta — `latest` has
# pointed at the newest beta since 0.1.0. Gating the channel on a stable/
# pre-release distinction the project does not have yet would freeze `latest`
# on 0.3.0 indefinitely, so the default is to move it.
#
# Opt out with LATEST=false when republishing an older version, where moving
# the channel would walk `just up` backwards.
function "tags" {
  params = [image]
  result = concat(
    VERSION != "" ? ["${DOCKER_USER}/${image}:${VERSION}"] : [],
    VERSION != "" && LATEST == "true" ? ["${DOCKER_USER}/${image}:latest"] : [],
  )
}

# One cache ref per entry, so a target's layers never collide with an unrelated
# one's. Both images point at the same backend ref on purpose — see `agent`.
function "cache_from" {
  params = [image]
  result = CACHE != "" ? [{ type = "registry", ref = "${CACHE}:${image}" }] : []
}

function "cache_to" {
  params = [image]
  result = CACHE != "" ? [{ type = "registry", ref = "${CACHE}:${image}", mode = "max" }] : []
}

target "_common" {
  context   = "."
  platforms = split(",", PLATFORMS)

  labels = {
    "org.opencontainers.image.source"   = "https://github.com/scylla-ops/scylla"
    "org.opencontainers.image.version"  = VERSION
    "org.opencontainers.image.revision" = GIT_SHA
  }
}

# Both binaries build from the workspace Dockerfile, which ends in one final
# stage per package; `target` picks the stage. Everything below it — the cooked
# dependency layer and the shared source layer — is identical for the two, so
# bake reuses it across both targets in a single run.
target "_backend" {
  inherits   = ["_common"]
  dockerfile = "Dockerfile"
}

target "control-plane" {
  inherits   = ["_backend"]
  target     = "scylla-control-plane"
  tags       = tags("scylla-control-plane")
  cache-from = cache_from("scylla-backend")
  cache-to   = cache_to("scylla-backend")
}

target "agent" {
  inherits   = ["_backend"]
  target     = "scylla-agent"
  tags       = tags("scylla-agent")
  # Same cache ref as the control plane on purpose: they share every layer up
  # to their own `cargo build -p`, and pointing both at one ref is what makes
  # the second image nearly free. The control plane's `ui` stage is extra
  # layers on that shared ref, not a competing one.
  cache-from = cache_from("scylla-backend")
  cache-to   = cache_to("scylla-backend")
}

group "release" {
  targets = ["control-plane", "agent"]
}

group "default" {
  targets = ["release"]
}
