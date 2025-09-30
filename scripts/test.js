#!/usr/bin/env node
const { execFileSync } = require("child_process");
const path = require("path");

try {
  const binaryPath = path.resolve(
    __dirname,
    "target",
    "debug",
    "rustzen-zipper"
  );
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  console.error("Failed to execute zipper binary:", err);
  process.exit(1);
}
