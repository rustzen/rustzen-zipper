const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const root = path.resolve(__dirname, "..");
const packageJson = require("../package.json");

function readCargoVersion() {
  const manifest = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const match = manifest.match(/^version = "([^"]+)"/m);
  if (!match) {
    throw new Error("Cargo.toml package version not found");
  }
  return match[1];
}

function assertVersionAligned() {
  const cargoVersion = readCargoVersion();
  if (packageJson.version !== cargoVersion) {
    throw new Error(
      `Version mismatch: package.json=${packageJson.version}, Cargo.toml=${cargoVersion}`,
    );
  }
}

function assertPackFiles() {
  const output = execFileSync("npm", ["pack", "--dry-run", "--json"], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      npm_config_cache: path.join(os.tmpdir(), "rustzen-zipper-npm-cache"),
    },
  });
  const [pack] = JSON.parse(output);
  const files = new Set(pack.files.map((file) => file.path));

  for (const required of [
    "index.js",
    "README.md",
    "scripts/install.js",
    "scripts/verify-package.js",
  ]) {
    if (!files.has(required)) {
      throw new Error(`npm package is missing ${required}`);
    }
  }
}

function assertWrapperCanRunBuiltBinary() {
  const isWin = process.platform === "win32";
  const binaryName = isWin ? "rustzen-zipper.exe" : "rustzen-zipper";
  const builtBinary = path.join(root, "target", "debug", binaryName);
  if (!fs.existsSync(builtBinary)) {
    throw new Error(`Built binary not found: ${builtBinary}. Run cargo build first.`);
  }

  const binDir = path.join(root, "bin");
  const wrapperBinary = path.join(binDir, binaryName);
  fs.mkdirSync(binDir, { recursive: true });

  const hadExistingBinary = fs.existsSync(wrapperBinary);
  const existingBinary = hadExistingBinary ? fs.readFileSync(wrapperBinary) : undefined;
  const existingMode = hadExistingBinary ? fs.statSync(wrapperBinary).mode : undefined;

  try {
    fs.copyFileSync(builtBinary, wrapperBinary);
    if (!isWin) {
      fs.chmodSync(wrapperBinary, 0o755);
    }

    const versionOutput = execFileSync("node", ["index.js", "--version"], {
      cwd: root,
      encoding: "utf8",
    });
    if (!versionOutput.includes(packageJson.version)) {
      throw new Error(`Wrapper version output does not include ${packageJson.version}`);
    }
  } finally {
    if (hadExistingBinary) {
      fs.writeFileSync(wrapperBinary, existingBinary);
      if (existingMode !== undefined) {
        fs.chmodSync(wrapperBinary, existingMode);
      }
    } else {
      fs.rmSync(wrapperBinary, { force: true });
    }
  }
}

assertVersionAligned();
assertPackFiles();
assertWrapperCanRunBuiltBinary();
console.log("Package verification passed.");
