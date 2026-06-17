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

(async () => {
  const { triple, ext } = detectTarget();
  const tag = `v${pkg.version}`;
  const repo = "rustzen/rustzen-zipper";
  const asset = `rustzen-zipper-${triple}${ext}`;
  const url = `https://github.com/${repo}/releases/download/${tag}/${asset}`;

  // 输出文件路径
  const out = path.join(
    BIN_PATH,
    ext ? "rustzen-zipper.exe" : "rustzen-zipper"
  );

  console.log(`Downloading ${asset} from ${url}`);
  await download(url, out);
  if (!ext) fs.chmodSync(out, 0o755);
  console.log(`Installed binary to ${out}`);
})();
