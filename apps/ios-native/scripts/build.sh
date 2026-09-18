#!/bin/bash
set -euo pipefail
APP_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$APP_ROOT/../.." && pwd)"
export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
BUILD_KIND="${1:-device}"
if [[ "$BUILD_KIND" == simulator ]]; then
  RUST_TARGET=aarch64-apple-ios-sim
  SDK=iphonesimulator
  DESTINATION='generic/platform=iOS Simulator'
else
  RUST_TARGET=aarch64-apple-ios
  SDK=iphoneos
  DESTINATION='generic/platform=iOS'
fi
export SDKROOT="$(xcrun --sdk "$SDK" --show-sdk-path)"
export CC="$(xcrun --find clang)"
cd "$REPO_ROOT"
cargo build -p wealthfolio-native-app --target "$RUST_TARGET"
mkdir -p "$APP_ROOT/Libraries/$SDK"
cp "${CARGO_TARGET_DIR:-$REPO_ROOT/target}/$RUST_TARGET/debug/libwealthfolio_native.a" "$APP_ROOT/Libraries/$SDK/"
xcodegen generate --spec "$APP_ROOT/project.yml" --project "$APP_ROOT"
SIGNING=(CODE_SIGNING_ALLOWED=NO)
# The Rust library is only built for arm64 simulators.
if [[ "$BUILD_KIND" == simulator ]]; then SIGNING+=(ARCHS=arm64); fi
if [[ -n "${WF_DEVELOPMENT_TEAM:-}" && "$BUILD_KIND" == device ]]; then
  SIGNING=("DEVELOPMENT_TEAM=$WF_DEVELOPMENT_TEAM" -allowProvisioningUpdates)
fi
xcodebuild -project "$APP_ROOT/WealthfolioNative.xcodeproj" -scheme WealthfolioNative \
  -configuration Debug -sdk "$SDK" -destination "$DESTINATION" \
  -derivedDataPath "$APP_ROOT/build/DerivedData" "${SIGNING[@]}" build
