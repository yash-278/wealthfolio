# Server and native client sync validation

As of 2026-09-13, on branch `codex/server-sync`, based on `2e89455e5`. All
database exercises used temporary synthetic databases.

| Check                                                                                                                   | Result                                                                                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo test -p wealthfolio-storage-sqlite sync::app_sync --lib`                                                         | 107 passed, zero failed                                                                                                                                                            |
| `cargo test -p wealthfolio-server --test server_sync_api`                                                               | Passed, including owner authentication, snapshot, uploads, retries, conflicts and ordinary web edits                                                                               |
| `cargo test -p wealthfolio-server --no-default-features --test server_sync_api`                                         | Passed with Connect features disabled                                                                                                                                              |
| `cargo clippy -p wealthfolio-storage-sqlite -p wealthfolio-server --all-targets -- -D warnings`                         | Passed                                                                                                                                                                             |
| `cargo test -p wealthfolio-server-sync --test round_trip`                                                               | 2 passed: authenticated two-device sync, offline/restart queue, lost acknowledgement retry, pause, web edits, conflict resolution, occupied-device refusal and endpoint validation |
| `cargo clippy -p wealthfolio-server-sync -p wealthfolio-storage-sqlite -p wealthfolio-app --all-targets -- -D warnings` | Passed                                                                                                                                                                             |
| `pnpm --filter frontend exec vitest run src/features/server-sync/settings.test.tsx`                                     | 3 passed: connect, pause, explicit discard confirmation                                                                                                                            |
| `pnpm type-check`                                                                                                       | Passed across the workspace                                                                                                                                                        |
| Targeted frontend ESLint                                                                                                | Passed                                                                                                                                                                             |
| `pnpm build:tauri` and `pnpm build`                                                                                     | Passed; existing bundle-size and sourcemap warnings remain                                                                                                                         |
| Disposable test-server smoke check                                                                                      | Started; owner login and sync opt-in returned HTTP 200                                                                                                                             |
| `cargo check -p wealthfolio-app`                                                                                        | Passed for the local macOS target                                                                                                                                                  |
| `cargo fmt --all -- --check`                                                                                            | Passed                                                                                                                                                                             |
| `git diff --check`                                                                                                      | Passed                                                                                                                                                                             |

The API test verifies that an uploaded goal is visible through the ordinary web
API, a subsequent web edit appears in the change feed, and retrying the earlier
upload remains a duplicate. Storage tests include concurrent conflicting edits,
preexisting data at opt-in, tombstones, cursor pagination, invalid payloads,
revision isolation, and injected journal failures that roll back domain writes.

No production database migration, commit, push or deployment was performed.

## iOS 27 Simulator validation

On 2026-09-13, `cargo check -p wealthfolio-app --target aarch64-apple-ios-sim`
passed. Xcode 27 RC (27A266a) built the unsigned debug app, which was installed
on the iPhone 17 Simulator running iOS 27 (24A434).

The first launch crashed in
`UIApplicationEvaluateRuntimeIssueForNoSceneLifecycleAdoption`. An eight-second
launch/process check reproduced the failure. Adding `UIApplicationSceneManifest`
with `UIApplicationSupportsMultipleScenes=true` in `Info.ios.plist` enables the
existing Tao 0.35.3 scene delegate. The same check passed after rebuilding and
reinstalling; a Simulator screenshot confirmed onboarding renders. Tao currently
gates scene support on that multiple-scenes flag, so iPad multi-window behavior
needs separate validation.

Xcode 27's default Swift Build engine also selected macOS headers for Tauri's
Swift iOS bridge. The scoped build script uses SwiftPM's native engine and
forwards the selected full Xcode path. The first corrected archive encountered
Tauri's existing-output rename failure; its app was moved to the Simulator
output and installed. The build script now removes that generated output before
a build.

Simulator launch is verified. Connection UI, Keychain session persistence and
end-to-end sync inside the Simulator have not yet been exercised. Physical
iPhone acceptance remains pending. See IOS-TESTING.md and README.md for the
remaining checks and unsupported data paths.

## Physical iPhone installation

On 2026-09-13, the signed debug archive passed
`codesign --verify --deep --strict` and installed successfully on the paired
iPhone 16 Pro running iOS 27. The provisioning profile included that phone.
Xcode Personal Team signing required removing the upstream Connect
associated-domains entitlement from the personal flavor; its config now sets the
mobile deep-link domains to an empty array.

The first launch request was denied because the device was locked. After unlock,
`devicectl device process launch` succeeded, and the user confirmed opening the
app. Physical launch is confirmed; phone/server pairing, Keychain persistence
and end-to-end sync have not yet been tested. No server deployment was
performed.

## Quick Add web-to-phone regression

The old activity source policy excluded QUICK_ADD from both the incremental
outbox and initial snapshots. A paired phone could reach the server cursor while
missing posted Quick Add transactions. The authenticated native-client
regression failed with "Quick Add web transaction must arrive on an
already-paired phone" before including that source. The coverage now checks
creation, update, deletion and initial download. A separate storage test covers
recovery of an omitted transaction into an existing pairing, unchanged ledger
data and repeated repair. Quick Add receipt and review objects remain outside
the sync dataset.
