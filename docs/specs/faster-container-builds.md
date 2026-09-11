# Faster Wealthfolio container builds

Status: draft, awaiting confirmation of the acceptance-test seams. This spec
covers follow-up optimization; the initial fork workflow is already added.

## Problem Statement

Small changes currently trigger lengthy Rust and frontend recompilation. Two
Railway builds ended after about 20 minutes without a compiler diagnosis. Lower
application optimization allowed a build to complete, but trades runtime
optimization for build time. The owner needs repeatable builds with measurable
cache reuse and a recoverable deployment path.

## Solution

Build the application in the GitHub fork, cache compiled dependencies separately
from application source, test the resulting image, and publish a
commit-addressed image to GHCR. Railway can later pull an explicitly selected
image after registry visibility and access are settled. Measure the improvement
before restoring normal release optimization.

## User Stories

1. As the owner, I want changes published only to my fork, so that upstream
   remains untouched.
2. As a developer, I want unchanged Rust dependencies reused, so that a small
   source edit does not rebuild them.
3. As a developer, I want frontend-only changes to reuse backend layers, so that
   independent components build independently.
4. As a developer, I want Rust-only changes to reuse frontend layers, so that
   frontend work is not repeated.
5. As a developer, I want dependency changes to invalidate the appropriate
   cache, so that stale binaries cannot be published.
6. As a developer, I want pinned toolchains and consistent build flags, so that
   cached and fresh builds use the same configuration.
7. As the owner, I want pull requests to build without publishing, so that
   proposed changes cannot replace a released image.
8. As the owner, I want only tested images published, so that startup failures
   block release.
9. As the owner, I want full commit identifiers and image digests recorded, so
   that every image is traceable.
10. As the owner, I want production secrets excluded from builds, so that image
    layers and logs do not contain credentials.
11. As the owner, I want measured cold and warm build timings, so that
    improvement is demonstrated.
12. As the owner, I want normal runtime optimization restored when practical, so
    that deployment constraints do not permanently reduce runtime performance.
13. As the owner, I want a visibility decision before Railway integration, so
    that registry access and cost requirements are understood.
14. As the owner, I want explicit image promotion, so that publication alone
    cannot update production.
15. As the owner, I want the previous image retained for rollback, so that a
    failed replacement is recoverable.
16. As the owner, I want existing accounts, transactions, volumes and settings
    preserved, so that a build-system change cannot alter my finances.

## Implementation Decisions

- GitHub Issues in the fork is the planning tracker. Upstream and the main
  task's working checkout are outside mutation scope.
- Keep build, publication, and Railway promotion separate. The initial workflow
  does not promote images.
- Use a single native Linux architecture initially. Verify compatibility with
  the existing Railway service before promotion.
- Use cargo-chef to separate Rust dependency compilation from application
  compilation. Pin the tool and keep recipe preparation, dependency cooking, and
  final compilation on the same target, toolchain, features, and profile.
- Separate frontend source inputs from backend inputs. Prepare frontend
  dependencies from lockfiles and workspace manifests first.
- Export intermediate BuildKit layers to the GitHub Actions cache. Do not assume
  cache mounts are persisted by layer-cache export.
- Publish only the smoke-tested image from main, using the workflow token, to
  the fork's GHCR namespace. Record its full commit tag and digest.
- Keep package visibility unresolved until the owner decides. Do not silently
  publish a public package or add Railway credentials.
- Benchmark normal release optimization before replacing the current
  reduced-optimization configuration. If tuning is still necessary, identify and
  measure the expensive component first.
- Preserve the application interfaces and data schema. No migrations or
  financial writes are part of build verification.

## Testing Decisions

- Test externally visible build and image behavior, not the presence of
  particular Dockerfile instructions.
- Prefer the existing Docker build boundary and disposable-container health
  endpoint as the primary integration seam.
- Measure one cold build, one unchanged rebuild, one frontend-only change, and
  one Rust-only change under identical runner and toolchain conditions. Record
  elapsed time, cache-hit stages, commit, image digest and compiler
  configuration.
- Verify dependency or toolchain changes invalidate affected layers while
  unaffected layers remain reusable. Cold builds must also succeed.
- Reuse the fork's existing frontend and Rust validation suites for application
  behavior.
- Fail publication when image startup or HTTP health fails. Smoke-test only
  disposable data with an ephemeral key and loopback-bound port.
- After separate production promotion authorization, check authenticated login,
  model discovery, and unchanged account and transaction snapshots. Use no real
  financial data in CI.
- The owner has been asked to confirm these acceptance seams; confirmation is
  pending.

## Out of Scope

New AI features, changing model routing, upstream publication, automatic
production deployment, new registry services, database migrations, bank imports,
multi-architecture builds, and guaranteed timing targets without measurements.

## Further Notes

The existing workflow adds GitHub layer-cache export but does not yet prove
source-change dependency reuse. Building in GitHub moves compilation away from
Railway; it does not by itself make compilation faster. Registry visibility,
Railway plan support and registry authentication must be decided before
promotion.
