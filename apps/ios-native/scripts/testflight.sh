#!/bin/bash
# Archives a Release build and uploads it to App Store Connect for TestFlight.
# Requires a paid Apple Developer Program team signed in to Xcode and an app
# record for the bundle identifier in App Store Connect.
set -euo pipefail
APP_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$APP_ROOT/../.." && pwd)"
export DEVELOPER_DIR="${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}"
: "${WF_DEVELOPMENT_TEAM:?Set WF_DEVELOPMENT_TEAM to your Apple Developer Program team ID}"
# Every upload needs a build number higher than the last one.
BUILD_NUMBER="${WF_BUILD_NUMBER:-$(date +%Y%m%d%H%M)}"
export SDKROOT="$(xcrun --sdk iphoneos --show-sdk-path)"
export CC="$(xcrun --find clang)"
cd "$REPO_ROOT"
cargo build -p wealthfolio-native-app --target aarch64-apple-ios --release
mkdir -p "$APP_ROOT/Libraries/iphoneos"
cp "${CARGO_TARGET_DIR:-$REPO_ROOT/target}/aarch64-apple-ios/release/libwealthfolio_native.a" "$APP_ROOT/Libraries/iphoneos/"
xcodegen generate --spec "$APP_ROOT/project.yml" --project "$APP_ROOT"
ARCHIVE="$APP_ROOT/build/WealthfolioNative.xcarchive"
xcodebuild -project "$APP_ROOT/WealthfolioNative.xcodeproj" -scheme WealthfolioNative \
  -configuration Release -destination 'generic/platform=iOS' -archivePath "$ARCHIVE" \
  "DEVELOPMENT_TEAM=$WF_DEVELOPMENT_TEAM" "CURRENT_PROJECT_VERSION=$BUILD_NUMBER" \
  -allowProvisioningUpdates archive
OPTIONS="$APP_ROOT/build/ExportOptions.plist"
cat > "$OPTIONS" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>method</key><string>app-store-connect</string>
	<key>destination</key><string>${WF_EXPORT_DESTINATION:-upload}</string>
	<key>teamID</key><string>$WF_DEVELOPMENT_TEAM</string>
	<key>signingStyle</key><string>automatic</string>
</dict>
</plist>
PLIST
xcodebuild -exportArchive -archivePath "$ARCHIVE" -exportOptionsPlist "$OPTIONS" \
  -exportPath "$APP_ROOT/build/TestFlight" -allowProvisioningUpdates
echo "Build $BUILD_NUMBER submitted. It appears in App Store Connect > TestFlight after processing."
