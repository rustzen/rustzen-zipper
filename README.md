# @rustzen/zipper

[![npm version](https://img.shields.io/npm/v/@rustzen/zipper.svg)](https://www.npmjs.com/package/@rustzen/zipper)
[![npm downloads](https://img.shields.io/npm/dm/@rustzen/zipper.svg)](https://www.npmjs.com/package/@rustzen/zipper)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

> 一个基于 Rust 的高性能 CLI 工具，用于快速压缩 dist 文件夹。

## 📋 目录

- [特性](#-特性)
- [安装](#-安装)
- [快速开始](#-快速开始)
- [配置来源](#-配置来源)
- [详细用法](#-详细用法)
- [时间格式说明](#-时间格式说明)
- [高级用法](#-高级用法)
- [故障排除](#️-故障排除)
- [帮助信息](#-帮助信息)
- [许可证](#-许可证)

## ✨ 特性

- 🚀 **高性能**：基于 Rust 构建，压缩速度极快
- 📦 **跨平台**：支持 Windows、macOS、Linux
- 🎯 **简单易用**：一行命令即可压缩
- ⚙️ **高度可配置**：支持自定义时间格式、源目录、压缩方法
- 🔧 **npm 集成**：完美集成到 npm 工作流

## 🧭 命令入口（当前有效）

发布后的包只提供 `rz-zip` 命令（由 `package.json` 的 `bin` 配置定义）。

- `rz-zip`：默认打包命令，直接执行 `rz-zip [选项]`。
- `rz-zip unpack`：解压命令（需提供 `--source`）。

## 配置来源

支持从 `.rzrc` / `.rzrc.json` / `package.json` 的 `rz-zip` 节点自动加载默认配置。

优先级：

1. CLI 显式参数（执行时传入）
2. `--config` 显式指定的路径（如 `--config .rzrc`）
3. 工作区内的 `.rzrc` / `.rzrc.json` / `package.json`
4. CLI 默认值

```bash
# .rzrc
{
  "source": "./dist",
  "output_dir": "./artifacts",
  "format": "%Y%m%d-%H%M%S",
  "compression": "deflated",
  "level": 9,
  "excludes": [".git", "node_modules", "*.map"],
  "base_dir": "dist",
  "strip_prefix": "build/dist",
  "sha256": true,
  "no_prefix": false,
  "overwrite": "overwrite",
  "dry_run": false,
  "quiet": false,
  "verbose": false
}
```

支持通过 `--config .rzrc` 显式指定，或在 `package.json` 使用顶层 `rz-zip` 字段。

```json
{
  "rz-zip": {
    "source": "./dist",
    "output_dir": "./artifacts",
    "format": "%Y%m%d-%H%M%S",
    "compression": "deflated",
    "level": 9,
    "excludes": [".git", "node_modules", "*.map"],
    "no_prefix": false,
    "overwrite": "overwrite"
  }
}
```

## 📦 安装

安装脚本会根据当前平台从 GitHub Release 下载 `rustzen-zipper-<target-triple>` 二进制，并放到包内的 `bin/rustzen-zipper`（Windows 为 `bin/rustzen-zipper.exe`）。支持的发布资产为：

- `rustzen-zipper-aarch64-apple-darwin`
- `rustzen-zipper-x86_64-apple-darwin`
- `rustzen-zipper-x86_64-unknown-linux-gnu`
- `rustzen-zipper-x86_64-pc-windows-msvc.exe`

发布 npm 包前必须先创建相同版本号的 GitHub Release，并上传上述平台资产；`package.json` 与 `Cargo.toml` 的版本号必须一致。

### 全局安装

```bash
npm install -g @rustzen/zipper
```

### 项目依赖

```bash
# npm
npm install -D @rustzen/zipper

# pnpm
pnpm add -D @rustzen/zipper

# yarn
yarn add -D @rustzen/zipper
```

## 🚀 快速开始

### 基本用法

```bash
# 压缩当前目录下的 dist 文件夹
rz-zip

# 输出：dist-20240928-1430.zip
```

### 在 package.json 中使用

```json
{
  "scripts": {
    "build": "vite build",
    "postbuild": "rz-zip"
  }
}
```

## 📖 详细用法

### 命令行参数

| 参数 | 长参数          | 说明                        | 默认值        |
| ---- | --------------- | --------------------------- | ------------- |
| `-s` | `--source`      | 源目录路径                  | `./dist`      |
| `-o` | `--output`      | 输出文件名（不含扩展名）    | 源文件名称    |
| `-f` | `--format`      | 时间格式                    | `%Y%m%d-%H%M` |
| `-l` | `--level`       | 压缩级别 `0-9`（仅 deflated）| `6`         |
| `-c` | `--compression` | 压缩方法 `stored`/`deflated` | `deflated`    |
| `-d` | `--output-dir`  | 输出目录（自动创建）        | `.`           |
| `-x` | `--exclude`     | 排除路径（glob，可重复）    | 无            |
| `-i` | `--include`     | 白名单路径（glob，可重复）  | 无            |
| `--config` |             | 配置文件路径（`--config .rzrc`） | none      |
|      | `--prefix`      | 保留源目录名（覆盖配置文件 `no_prefix`） | false |
|      | `--no-prefix`   | 不保留源目录名写入 zip      | false         |
|      | `--base-dir`    | 覆盖压缩包根目录名          | 源目录名      |
|      | `--strip-prefix`| 从条目路径移除前缀          | 无            |
|      | `--sha256`      | 额外输出 `<name>.zip.sha256` | false         |
|      | `--no-sha256`   | 不输出 `<name>.zip.sha256`   | false         |
|      | `--overwrite`   | 覆盖策略：`overwrite` `skip` `error` | `overwrite` |
|      | `--dry-run`     | 仅预览包含/排除内容，不创建压缩包 | false     |
|      | `--no-dry-run`  | 强制创建压缩包（覆盖 `dry_run`） | false      |
| `-q` | `--quiet`       | 只输出错误信息             | false         |
|      | `--no-quiet`    | 允许正常输出               | false         |
| `-v` | `--verbose`     | 详细日志                   | false         |
|      | `--no-verbose`  | 关闭详细日志               | false         |

### 使用示例

#### 1. 自定义源目录

```bash
# 压缩 build 目录
rz-zip -s ./build

# 压缩 public 目录
rz-zip --source ./public
```

#### 2. 自定义输出文件名

```bash
rz-zip -o myapp
# 输出为 myapp-20240928-1430.zip

rz-zip --output deploy
# 输出为 deploy-20240928-1430.zip
```

#### 3. 自定义时间格式

```bash
# 年月日时分
rz-zip -f "%Y%m%d%H%M"
# 输出：dist-202409281430.zip

# 带分隔符
rz-zip -f "%Y-%m-%d_%H-%M"
# 输出：dist-2024-09-28_14-30.zip
```

#### 4. 压缩方法

```bash
# 无压缩
rz-zip -c stored

# 标准压缩
rz-zip -c deflated
```

#### 5. 组合使用

```bash
# 完整示例
rz-zip -s ./build -o deploy -f "%Y%m%d" -c deflated -l 9 --no-prefix
# 输出：deploy-20240928.zip
```

#### 6. 排除指定路径

```bash
rz-zip -s ./dist -x node_modules -x .git
rz-zip -s ./dist -o release -x ".map" -x "tmp/"
```

#### 7. 指定输出目录

```bash
rz-zip -s ./dist -d ./artifacts
rz-zip -s ./dist -o release -d ./output
```

#### 8. 使用统一配置文件

```bash
rz-zip --config .rzrc -q
rz-zip -s ./dist --config .rzrc
```

#### 9. 使用自定义根目录与前缀裁剪

```bash
rz-zip -s ./dist --base-dir release
rz-zip -s ./dist --strip-prefix "build/dist" --no-prefix
```

#### 10. 生成校验和

```bash
rz-zip -s ./dist -o app --sha256
```

会额外生成：

```text
app-20260603-1200.zip.sha256
```

### 参数说明补充

- `-c/--compression` 现在会做参数校验，只允许 `stored` 或 `deflated`。
- `-d/--output-dir` 用于指定压缩包目录，目录不存在会自动创建。
- `-l/--level` 压缩级别支持 `0-9`；`stored` 模式会忽略该参数。
- `-i/--include` 为白名单匹配规则（同样支持 glob）；若设置则只会打包命中项。
- `--no-prefix` 关闭后不保留源目录名（压缩包内直接是相对路径）。
- `--overwrite` 支持三种策略：`overwrite`（默认覆盖）、`skip`（文件存在时跳过）和 `error`（文件存在时报错退出）。
- `--dry-run` 仅展示包含/排除预览并输出统计摘要，不会生成压缩包。
- `-q/--quiet` 与 `-v/--verbose` 可用于减噪/排障。
- `--base-dir` 直接替换 zip 根目录名，即使不带 `--no-prefix` 也会按该值写入。
- `--strip-prefix` 在 `--include/--exclude` 匹配之后再应用，用于移除条目相对路径前缀。
- `--sha256` 在打包完成后同时生成同目录 `<zip>.sha256`，内容格式为：`<sha256><空格><空格><zip文件名>`.

## 🕒 时间格式说明

基于 [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) 库的格式：

| 格式 | 说明         | 示例 |
| ---- | ------------ | ---- |
| `%Y` | 4 位年份     | 2024 |
| `%m` | 月份 (01-12) | 09   |
| `%d` | 日期 (01-31) | 28   |
| `%H` | 小时 (00-23) | 14   |
| `%M` | 分钟 (00-59) | 30   |
| `%S` | 秒 (00-59)   | 45   |

### 常用格式示例

```bash
# 年月日时分
rz-zip -f "%Y%m%d%H%M"
# 输出：dist-202409281430.zip

# 年月日时分秒
rz-zip -f "%Y%m%d%H%M%S"
# 输出：dist-20240928143045.zip

# 带分隔符
rz-zip -f "%Y-%m-%d_%H-%M"
# 输出：dist-2024-09-28_14-30.zip

# 简单日期
rz-zip -f "%d%m%Y"
# 输出：dist-28092024.zip
```

## 🔧 高级用法

### CI/CD 集成

```yaml
# GitHub Actions
- name: Build and zip
  run: |
    npm run build
    rz-zip -f "build_%Y%m%d_%H%M"
```

### 多环境部署

```json
{
  "scripts": {
    "build:dev": "vite build --mode development",
    "build:prod": "vite build --mode production",
    "zip:dev": "rz-zip -s ./dist -o dev -f dev_%Y%m%d",
    "zip:prod": "rz-zip -s ./dist -o prod -f prod_%Y%m%d"
  }
}
```

### 自动化脚本

```bash
#!/bin/bash
# deploy.sh
echo "Building project..."
npm run build

echo "Creating deployment package..."
rz-zip -f "deploy_%Y%m%d_%H%M"

echo "Package created successfully!"
```

### 📋 输出文件

- **默认命名**：`{output}-{timestamp}.zip`
- **位置**：当前工作目录
- **内容**：保留原始目录结构，支持空目录
- **权限**：适当的文件权限设置

## 🛠️ 故障排除

### 二进制未找到

安装阶段需要访问 GitHub Release。如果处在离线环境或企业网络限制下，请先确认能够访问 `https://github.com/rustzen/rustzen-zipper/releases`，并确认当前 npm 版本对应的 Release 资产已经上传。

```bash
# 重新安装
npm uninstall -g @rustzen/zipper
npm install -g @rustzen/zipper
```

### 源目录不存在

```bash
# 检查目录
ls -la ./dist
# 或指定正确的路径
rz-zip -s ./正确的目录路径
```

### 压缩方法不支持

```bash
# 检查支持的压缩方法
rz-zip --help
# 使用默认的 deflated 方法
rz-zip -c stored
```

## 📚 帮助信息

```bash
# 显示帮助
rz-zip --help
rz-zip -h

# 显示版本
rz-zip --version
rz-zip -V
```

## 📄 许可证

MIT License

---

**注意**：确保源目录存在且包含文件，否则压缩会失败。
