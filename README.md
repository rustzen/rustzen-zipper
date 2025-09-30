# @rustzen/zipper

一个基于 Rust 的高性能 CLI 工具，用于快速压缩 dist 文件夹。

## ✨ 特性

- 🚀 **高性能**：基于 Rust 构建，压缩速度极快
- 📦 **跨平台**：支持 Windows、macOS、Linux
- 🎯 **简单易用**：一行命令即可压缩
- ⚙️ **高度可配置**：支持自定义时间格式、源目录、压缩方法
- 🔧 **npm 集成**：完美集成到 npm 工作流

## 📦 安装

### 全局安装

```bash
npm install -g @rustzen/zipper
```

### 项目依赖

```bash
npm install -D @rustzen/zipper
# 或
pnpm add -D @rustzen/zipper
```

## 🚀 快速开始

### 基本用法

```bash
# 压缩当前目录下的 dist 文件夹
zipper

# 输出：dist-20240928-1430.zip
```

### 在 package.json 中使用

```json
{
  "scripts": {
    "build": "vite build",
    "postbuild": "zipper",
    "deploy": "npm run build && zipper -f=%Y%m%d"
  }
}
```

## 📖 详细用法

### 命令行参数

| 参数 | 长参数          | 说明                     | 默认值        |
| ---- | --------------- | ------------------------ | ------------- |
| `-s` | `--source`      | 源目录路径               | `./dist`      |
| `-o` | `--output`      | 输出文件名（不含扩展名） | `dist`        |
| `-f` | `--format`      | 时间格式                 | `%Y%m%d-%H%M` |
| `-c` | `--compression` | 压缩方法                 | `stored`      |

### 使用示例

#### 1. 自定义源目录

```bash
# 压缩 build 目录
zipper -s ./build

# 压缩 public 目录
zipper --source ./public
```

#### 2. 自定义输出文件名

```bash
# 输出为 myapp-20240928-1430.zip
zipper -o myapp

# 输出为 deploy-20240928-1430.zip
zipper --output deploy
```

#### 3. 自定义时间格式

```bash
# 年月日时分
zipper -f "%Y%m%d%H%M"
# 输出：dist-202409281430.zip

# 带分隔符
zipper -f "%Y-%m-%d_%H-%M"
# 输出：dist-2024-09-28_14-30.zip

# 简单日期
zipper -f "%d%m%Y"
# 输出：dist-28092024.zip
```

#### 4. 压缩方法

```bash
# 无压缩（默认，最快）
zipper -c stored

# 标准压缩
zipper -c deflated
```

#### 5. 组合使用

```bash
# 完整示例
zipper -s ./build -o deploy -f "%Y%m%d" -c deflated
# 输出：deploy-20240928.zip
```

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
zipper -f "%Y%m%d%H%M"
# 输出：dist-202409281430.zip

# 年月日时分秒
zipper -f "%Y%m%d%H%M%S"
# 输出：dist-20240928143045.zip

# 带分隔符
zipper -f "%Y-%m-%d_%H-%M"
# 输出：dist-2024-09-28_14-30.zip

# 简单日期
zipper -f "%d%m%Y"
# 输出：dist-28092024.zip
```

## 🔧 高级用法

### CI/CD 集成

```yaml
# GitHub Actions
- name: Build and zip
  run: |
    npm run build
    zipper -f "build_%Y%m%d_%H%M"
```

### 多环境部署

```json
{
  "scripts": {
    "build:dev": "vite build --mode development",
    "build:prod": "vite build --mode production",
    "zip:dev": "zipper -s ./dist -o dev -f dev_%Y%m%d",
    "zip:prod": "zipper -s ./dist -o prod -f prod_%Y%m%d"
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
zipper -f "deploy_%Y%m%d_%H%M"

echo "Package created successfully!"
```

## 📋 输出文件

- **默认命名**：`{output}-{timestamp}.zip`
- **位置**：当前工作目录
- **内容**：保留原始目录结构，支持空目录
- **权限**：适当的文件权限设置

## 🛠️ 故障排除

### 二进制未找到

```bash
# 重新安装
npm uninstall -g @rustzen/zipper
npm install -g @rustzen/zipper
```

### 权限问题

```bash
# 检查文件权限
ls -la bin/
chmod +x bin/rustzen-zipper
```

### 源目录不存在

```bash
# 检查目录
ls -la ./dist
# 或指定正确的路径
zipper -s ./正确的目录路径
```

### 压缩方法不支持

```bash
# 检查支持的压缩方法
zipper --help
# 使用默认的 stored 方法
zipper -c stored
```

## 📚 帮助信息

```bash
# 显示帮助
zipper --help
zipper -h

# 显示版本
zipper --version
zipper -V
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

---

**注意**：确保源目录存在且包含文件，否则压缩会失败。
