#!/usr/bin/env node
const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const isWin = process.platform === "win32";
const binName = isWin ? "rustzen-zipper.exe" : "rustzen-zipper";
const binExec = path.resolve(__dirname, "bin", binName);

if (!fs.existsSync(binExec)) {
  console.error(`Binary not found: ${binExec}`);
  process.exit(1);
}
execFileSync(binExec, process.argv.slice(2), { stdio: "inherit" });
