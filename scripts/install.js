const https = require("https");
const fs = require("fs");
const path = require("path");
const pkg = require("../package.json");

const BIN_PATH = path.resolve(__dirname, "..", "bin");
// 创建执行文件存放目录
if (!fs.existsSync(BIN_PATH)) {
  fs.mkdirSync(BIN_PATH, { recursive: true });
}

function detectTarget() {
  const { platform, arch } = process;
  if (platform === "darwin") {
    if (arch === "arm64") return { triple: "aarch64-apple-darwin", ext: "" };
    if (arch === "x64") return { triple: "x86_64-apple-darwin", ext: "" };
    throw new Error(`Unsupported darwin arch: ${arch}`);
  }
  if (platform === "linux")
    return { triple: "x86_64-unknown-linux-gnu", ext: "" };
  if (platform === "win32")
    return { triple: "x86_64-pc-windows-msvc", ext: ".exe" };
  throw new Error(`Unsupported platform: ${platform}`);
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        )
          return resolve(download(res.headers.location, dest));
        if (res.statusCode !== 200)
          return reject(new Error(`HTTP ${res.statusCode} ${url}`));
        const out = fs.createWriteStream(dest, { mode: 0o755 });
        res.pipe(out);
        out.on("finish", () => out.close(resolve));
        out.on("error", reject);
      })
      .on("error", reject);
  });
}

// 检查二进制文件是否存在且可执行
function checkBinaryExists(binaryName) {
  try {
    const binaryPath = path.join(BIN_PATH, binaryName);
    if (fs.existsSync(binaryPath)) {
      return true;
    }
  } catch (error) {
    // 文件存在但不可执行，需要重新下载
    console.log(`Binary exists but not executable, will re-download`);
  }
  return false;
}

// 复制文件
function copyFile(src, dest) {
  try {
    const srcPath = path.join(BIN_PATH, src);
    fs.copyFileSync(srcPath, dest);
    console.log(`Copied binary from ${srcPath} to ${dest}`);
  } catch (error) {
    throw new Error(`Failed to copy binary: ${error.message}`);
  }
}

(async () => {
  const { triple, ext } = detectTarget();
  const tag = `v${pkg.version}`;
  const repo = "idaibin/rustzen-zipper";
  const asset = `rustzen-zipper-${triple}${ext}`;
  const url = `https://github.com/${repo}/releases/download/${tag}/${asset}`;

  // 输出文件路径
  const out = path.join(
    BIN_PATH,
    ext ? "rustzen-zipper.exe" : "rustzen-zipper"
  );

  // 检查二进制文件是否已存在
  if (checkBinaryExists(asset)) {
    console.log(`Binary already exists: ${out}`);
    // 复制到统一文件名
    copyFile(asset, out);
    if (!ext) fs.chmodSync(out, 0o755);
    return;
  }
  console.log(`Downloading ${asset} from ${url}`);
  await download(url, out);
  if (!ext) fs.chmodSync(out, 0o755);
  console.log(`Installed binary to ${out}`);
})();
