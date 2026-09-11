# Fork container builds

The Container image workflow builds Linux amd64 with GitHub Actions and tests
startup plus HTTP health against disposable local data. Pull requests only build
and test. Pushes to main publish the tested image to the fork's GHCR namespace
with a full commit SHA tag. The run summary records the image digest.

No Railway deployment is triggered. No production or AWS credentials are used.
GHCR publication uses the workflow token. Package visibility is not changed by
the workflow; decide visibility and Railway registry access before deployment.

BuildKit exports intermediate layers to the GitHub Actions cache. The existing
Dockerfile still invalidates dependency build layers on source edits. Separating
those layers, benchmarking cold and warm builds, and restoring normal release
optimization are follow-up work, not benefits already verified here.

The build is limited to 60 minutes. A failed build or smoke test cannot publish
an image. The smoke test disables authentication only in its disposable container,
which is bound to the runner's loopback interface.
