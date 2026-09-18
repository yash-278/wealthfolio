# Testing own-server sync on iOS

The shared native client has been tested against a disposable authenticated
server with two independent local databases. The unsigned app now builds with
Xcode 27 and launches on the iPhone 17 Simulator. End-to-end sync inside
Simulator remains to be exercised. On 2026-09-13, the Simulator Rust target
passed compilation. Xcode 27 RC is available in `/Applications/Xcode.app` and
recognizes the running iPhone 17 with iOS 27. The older Xcode 26.6 copy has been
replaced. Use Xcode 27 for this test. Xcode 27 opens simulators in Device Hub
(`Xcode.app/Contents/Applications/DeviceHub.app`).

## Prerequisites

Open full Xcode after installation and complete its first-launch setup,
including an iOS Simulator runtime. Verify `xcodebuild -version` and
`xcrun simctl list devices available`. If Command Line Tools remains selected,
select the full Xcode developer directory with `xcode-select` first.

An isolated Rust 1.95.0 toolchain is installed at
`/private/tmp/wealthfolio-ios-toolchain`, with both `aarch64-apple-ios` and
`aarch64-apple-ios-sim`. The system Homebrew toolchain and shell profiles were
not changed. For this testing session, set:

```sh
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export RUSTUP_HOME=/private/tmp/wealthfolio-ios-toolchain/rustup
export CARGO_HOME=/private/tmp/wealthfolio-ios-toolchain/cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

Temporary toolchain files may be removed by the OS; reinstall the two targets if
needed. Use the repository's pinned pnpm 10.33.4. See
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) and
[Apple device and Simulator setup](https://developer.apple.com/documentation/Xcode/running-your-app-on-simulated-or-physical-devices).

## Disposable Simulator test

Run commands from the repository root. Start the synthetic server in a terminal:

```sh
cargo run -p wealthfolio-server-sync --example test_server
```

It listens only on `http://127.0.0.1:8089`, with test password `simulator-only`
and a seeded goal. Its database is temporary and is discarded on shutdown. Do
not use this fixture for real financial data. Restarting it creates a different
server identity, so use a fresh disposable app installation for each server run.

Initialize the personal iOS flavor, then launch a Simulator:

```sh
node apps/tauri/scripts/sync-ios-composer-icon.mjs
pnpm tauri ios init --config apps/tauri/tauri.personal-ios.conf.json
pnpm tauri ios dev --config apps/tauri/tauri.personal-ios.conf.json
```

The config selects `com.yashkadam.wealthfolio.personal`, display name
Wealthfolio Personal, and removes the upstream signing team. Initialization was
exercised with XcodeGen and CocoaPods. The tracked Xcode project template also
uses the personal identity because `ios init --config` does not replace identity
settings in an existing template. Use `--open` on `ios dev` to open the
generated project in Xcode for debugging.

1. Start with a fresh personal app and open Settings → Your server before adding
   local records. Enter the loopback URL and synthetic password above.
2. Confirm the seeded goal appears after Connect and download.
3. Change its title in the app, select Sync now, and check the web UI at the
   same loopback origin. The fixture serves the latest frontend `dist` if
   available (`pnpm build` produces the web bundle).
4. Pause sync, make an edit, terminate/relaunch the app, and confirm the pending
   count persists. Resume and verify the edit reaches the server.
5. Pause the app, edit the same goal on the web, then edit locally. Resume sync.
   Confirm the local version stays visible until Use server version and the
   discard confirmation are selected.
6. Stop the server, make an edit and run sync. Confirm the error and pending
   count. The automated integration test covers reconnect/retry without
   replacing the fixture database; do not restart this disposable fixture
   expecting it to have the same identity.
7. Verify sign-in persistence after app relaunch, scrolling and controls on a
   narrow iPhone, and foreground refresh after switching apps.

## Xcode 27 Simulator build

After initialization, build the personal Simulator app with:

```sh
node apps/tauri/scripts/build-personal-ios-simulator.mjs
```

The script scopes two compatibility settings to this build: it forwards the
selected full Xcode path to Tauri's archive subprocess, and uses SwiftPM's
native build engine for `swift-rs` 1.0.7. Xcode 27's default Swift Build engine
selected macOS headers while compiling Tauri's iOS bridge with the dependency's
existing cross-compilation flags. The script changes no global Xcode selection
or Swift installation. It builds an unsigned debug app with embedded frontend
assets.

## Physical iPhone

Use the personal bundle identifier and your own signing team in Xcode. The
personal config disables upstream Connect associated-domain links, which a
Personal Team cannot provision. Own-server sync does not need those links.
Enable Developer Mode on the phone when Xcode requests it. A phone cannot reach
the Mac server through `127.0.0.1`; use an HTTPS test server reachable from the
phone, with this branch's server implementation deployed. The existing
production server has not been updated by this task. Physical-device testing
should use synthetic records until bootstrap, Keychain persistence and reconnect
behavior have been confirmed.

Widgets, background scheduling, Quick Add capture receipts/review data and
unsupported special upload entities are outside this test's acceptance scope.

## iOS 27 launch requirement

`Info.ios.plist` enables `UIApplicationSceneManifest` with
`UIApplicationSupportsMultipleScenes=true`. Tao 0.35.3 only registers its scene
delegate when that flag is true. Without it, iOS 27 terminates the app before
onboarding. This was reproduced and corrected in Simulator. iPad multi-window
behavior has not been tested. See
[Apple's scene lifecycle guidance](https://developer.apple.com/documentation/uikit/transitioning-to-the-uikit-scene-based-life-cycle).

A signed physical-device build was installed on the paired iPhone 16 Pro with
iOS 27 on 2026-09-13. Developer Mode and provisioning-device eligibility were
verified. The phone must be unlocked for `devicectl device process launch` to
open the app. The Simulator and device builds are separate artifacts.
