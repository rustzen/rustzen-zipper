#!/usr/bin/env node
const { execFileSync } = require("child_process");
const path = require("path");

try {
  const binaryName = process.platform === "win32" ? "rustzen-zipper.exe" : "rustzen-zipper";
  const binaryPath = path.resolve(
    __dirname,
    "..",
    "target",
    "debug",
    binaryName
  );
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  console.error("Failed to execute rz-zip binary:", err);
  process.exit(1);
}
