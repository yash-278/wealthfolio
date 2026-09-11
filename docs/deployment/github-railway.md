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

Select a published immutable image in Railway after its run succeeds. Keep the
/data volume and existing environment variables attached. Record the previous
image digest before promotion; rollback selects that image again. Publication by
itself does not change production. No production secrets are used in CI.
