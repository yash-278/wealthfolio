# GitHub container delivery

GitHub Actions builds a single linux/amd64 image. A disposable-container health
check must pass before publication. Pull requests build only. Main pushes and
explicit workflow dispatches publish `ghcr.io/yash-278/wealthfolio:sha-COMMIT`.
The run summary records the immutable digest. The image contains application
code and assets; credentials and financial storage belong only to Railway.

The Dockerfile separates pnpm installation, frontend source, cargo-chef recipe,
compiled Rust dependencies, and backend source. BuildKit exports intermediate
layers to the GitHub Actions cache. It does not rely on persistent cache mounts.
Source-only changes can reuse compiled dependency layers; Rust-only changes do
not copy frontend inputs and frontend-only changes do not copy backend inputs.
The initial cold build remains expensive. Lockfile, manifest, toolchain, or
compiler-flag changes intentionally invalidate affected dependency layers.

Enable the workflow's benchmark input to measure unchanged, frontend-only and
Rust-only rebuilds using the initial build's exported local layer cache. Timings
and plain build logs are saved as an artifact. Inspect CACHED stages alongside
elapsed time; runner variation means timings alone do not establish cache reuse.
The initial action duration and image digest are in the same workflow run.

Deployment preserves the /data volume and existing environment variables. Build
and test jobs do not receive production credentials.

The `Promote Railway image` workflow accepts an immutable digest and uses the
repository secret `RAILWAY_TOKEN`. That token must be scoped to the Wealthfolio
project and production environment. It is a deployment credential, not an
application or AWS key. The workflow records the prior deployment, pulls the
public image, selects it in Railway, and waits for a new successful deployment.

Successful pushes to main automatically promote the exact tested image digest to
Railway. Pull requests never deploy. The manual promotion workflow remains
available for rollback. The caller passes the repository secret RAILWAY_TOKEN
explicitly to the reusable workflow. Keep a single copy at repository scope; a
same-named environment secret can override it. The production environment still
records deployment history. Never use an account-wide credential for this
workflow.
