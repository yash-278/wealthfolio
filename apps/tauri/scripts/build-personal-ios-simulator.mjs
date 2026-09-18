// Xcode 27 defaults SwiftPM to Swift Build, which ignores swift-rs 1.0.7's
// iOS cross-compilation flags. Scope the native-engine workaround to this build.
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const developer = process.env.DEVELOPER_DIR || execFileSync("xcode-select", ["-p"], { encoding: "utf8" }).trim();
const env = { ...process.env, DEVELOPER_DIR: developer };
const swift = execFileSync("xcrun", ["--find", "swift"], { env, encoding: "utf8" }).trim();
const wrappers = mkdtempSync(path.join(tmpdir(), "wealthfolio-ios-tools-"));
const quote = (value) => "'" + value.replaceAll("'", "'\\''") + "'";
try {
  // Tauri cannot rename the new Simulator bundle over an existing nonempty one.
  // This directory contains build output only, never the installed app's data.
  rmSync(path.join(root, "apps/tauri/gen/apple/build/arm64-sim/Wealthfolio Personal.app"), { recursive: true, force: true });
  writeFileSync(path.join(wrappers, "xcodebuild"), `#!/bin/sh
export DEVELOPER_DIR=${quote(developer)}
exec ${quote(path.join(developer, "usr/bin/xcodebuild"))} "$@"
`, { mode: 0o755 });
  writeFileSync(path.join(wrappers, "swift"), `#!/bin/sh
export DEVELOPER_DIR=${quote(developer)}
if [ "$1" = build ]; then
  exec ${quote(swift)} "$@" --build-system native
fi
exec ${quote(swift)} "$@"
`, { mode: 0o755 });
  const result = spawnSync("pnpm", ["tauri", "ios", "build", "--debug", "--target", "aarch64-sim", "--no-sign", "--ci", "--config", "apps/tauri/tauri.personal-ios.conf.json", ...process.argv.slice(2)], {
    cwd: root, env: { ...env, PATH: `${wrappers}:${env.PATH}` }, stdio: "inherit",
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
} finally {
  rmSync(wrappers, { recursive: true, force: true });
}
