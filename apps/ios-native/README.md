# Wealthfolio Native

Standalone SwiftUI iOS client, minimum iOS 26. The existing Tauri client is
retained as the reference. This is an initial implementation, not full feature
parity.

## Architecture

SwiftUI calls `wealthfolio-native-app` through a small C ABI. The Rust library
calls the existing Axum request handlers in process using `Router::oneshot`. No
socket is bound, no HTTP server process is started, and no WKWebView or HTML is
rendered. The same core services, database migrations, domain events and write
actor perform validation, calculations and persistence. Network access is needed
for optional server sync, market prices and AI providers, not for local
portfolio reads/edits.

The only change to server initialization is an optional SecretStore injection.
The normal server entry point retains its encrypted-file store; the native
client injects Keychain. No deployment or API/protocol change is required.

The separate bundle identifier is `com.yashkadam.wealthfolio.native`. Its SQLite
database is in its own Application Support directory. It does not read or
overwrite the reference app's database. User preferences are initialized in
native onboarding; the current sync protocol intentionally keeps display
currency/timezone local.

## Build

Requires Xcode 26 or later, Rust with `aarch64-apple-ios` or
`aarch64-apple-ios-sim`, and XcodeGen. From the repository root:

```sh
bash apps/ios-native/scripts/build.sh device
bash apps/ios-native/scripts/build.sh simulator
```

Set `WF_DEVELOPMENT_TEAM` to the selected Xcode team to sign a device build.
Without it, the script compiles an unsigned build. `CARGO_TARGET_DIR`,
`CARGO_HOME`, `RUSTUP_HOME` and `DEVELOPER_DIR` can be supplied by the build
environment.

## TestFlight

Requires a paid Apple Developer Program team signed in to Xcode, and an app
record for the bundle identifier in App Store Connect.

```sh
WF_DEVELOPMENT_TEAM=<team id> bash apps/ios-native/scripts/testflight.sh
```

The script builds the Rust library in release mode, archives a Release build
with a timestamp build number (`WF_BUILD_NUMBER` overrides it) and uploads it.
Set `WF_EXPORT_DESTINATION=export` to write the `.ipa` to `build/TestFlight`
without uploading. App Store Connect asks the export compliance question for
each build; the binary bundles ChaCha20-Poly1305, X25519 and Argon2 in addition
to system TLS. `scripts/make-icon.swift` regenerates the app icon.

## Checks

```sh
cargo test -p wealthfolio-native-app --test offline_contract
cargo test -p wealthfolio-native-app --test sync_contract
```

```sh
xcodebuild -project apps/ios-native/WealthfolioNative.xcodeproj -scheme WealthfolioNativeUITests \
  -destination 'platform=iOS Simulator,name=iPhone 17' ARCHS=arm64 test
```

Run the simulator build first. The screen tour expects a fresh install and
attaches a screenshot per screen to the result bundle.

The offline contract verifies account/transaction creation, canonical rejection
of invalid input, persisted data after reopening, initial screen reads and
absence of a secrets file. The sync contract uses an authenticated synthetic
local server and verifies snapshot pairing, queued offline edits, upload, and
web Quick Add download. Neither test accesses production or a personal
portfolio.

See `PARITY.md` before treating this app as a replacement for the reference
client.
