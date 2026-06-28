# Tauri 网页套壳模板

一个极简的 Tauri v2 桌面应用模板——将任何 HTML 页面打包成 Windows 桌面程序。

**特点：**
- Rust 层零业务逻辑，纯空壳
- 自带无边框窗口 + 仿原生标题栏（最小化/最大化/关闭）
- 已内置右键菜单禁用、文本选定禁用（可在 HTML 中移除）
- 构建时自动压缩 HTML，减小体积

## 目录结构

```
template/
├── dist/
│   └── index.html          ← 你的网页放这里
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         ← 入口（无需修改）
│   │   └── lib.rs          ← Tauri 空壳（无需修改）
│   ├── capabilities/
│   │   └── default.json    ← 窗口权限（无需修改）
│   ├── icons/              ← 应用图标
│   ├── Cargo.toml          ← Rust 项目配置
│   ├── build.rs            ← 构建脚本（无需修改）
│   └── tauri.conf.json     ← 窗口大小/标题等
├── package.json            ← Node 脚本
├── minify.cjs              ← HTML 压缩（构建时自动调用）
├── sync-version.cjs        ← 版本号同步（构建时自动调用）
└── .gitignore
```

## 前提条件

1. 安装 [Rust](https://rustup.rs/)
2. 安装 [Node.js](https://nodejs.org/)（≥ 18）
3. 安装 npm 依赖：

```bash
npm install
```

## 快速开始

### 1. 编写你的网页

直接修改或替换 `dist/index.html`。

模板中已包含一个带窗口控件的示例页面。如果你要完全用自己的 HTML，需要注意：

- **保留窗口控制栏**：`<div class="titlebar">` 及其子元素，以及尾部 `<script type="module">` 中的窗口控制逻辑。这是 Tauri 无边框窗口的拖拽和关闭/最小化/最大化功能。
- 或者你也可以在 `tauri.conf.json` 中将 `"decorations": false` 改为 `true`，使用系统原生标题栏（那样就可以删除 HTML 中的标题栏代码）。

### 2. 预览（实时刷新）

```bash
npm run tauri dev
```

修改 `dist/index.html`，窗口会自动刷新。

### 3. 构建 exe

```bash
npm run tauri build
```

构建产物在 `src-tauri/target/release/` 目录下。

## 自定义配置

需要修改的地方（按优先级）：

### 必改项

| 文件 | 字段 | 说明 |
|------|------|------|
| `tauri.conf.json` | `productName` | 应用名称 |
| `tauri.conf.json` | `identifier` | 唯一标识，建议 `com.xxx.yyy` |
| `tauri.conf.json` | `windows[0].title` | 窗口标题 |
| `tauri.conf.json` | `windows[0].width/height` | 窗口尺寸 |
| `Cargo.toml` | `name` | 内部名称（需与 bin.name、lib.name 一致） |
| `Cargo.toml` | `[package.metadata.tauri-winres]` 下所有字段 | Windows EXE 属性（右键文件→属性→详细信息） |
| `package.json` | `name` | npm 包名 |
| `dist/index.html` | `<title>`、`.titlebar-title` | 页面标题、标题栏文字 |

### 可选改动

| 文件 | 字段 | 说明 |
|------|------|------|
| `tauri.conf.json` | `version` | 版本号（同步脚本会自动同步到其他文件） |
| `tauri.conf.json` | `windows[0].resizable` | 是否可调整大小 |
| `tauri.conf.json` | `windows[0].decorations` | `false`=无边框+自定义标题栏，`true`=系统原生标题栏 |
| `src-tauri/icons/` | 所有图标 | 替换为你自己的应用图标 |

### 版本号规则

修改版本号**只需改一处**：`tauri.conf.json` 中的 `version` 字段。

构建时 `sync-version.cjs` 会自动将版本号同步到 `package.json` 和 `Cargo.toml`。

## 隐藏 Web 特征

模板已内置以下措施防止暴露网页本质：

- `<body oncontextmenu="return false;">` — 禁用右键菜单
- `document.addEventListener('contextmenu', ...)` — JS 层拦截右键
- `body { user-select: none }` — 禁用文本选定

如需启用，删除对应代码即可。

## 添加窗口时显示白屏？

这是设计如此——`tauri.conf.json` 中 `"visible": false`，页面加载完成后 JS 调用 `plugin:window|show` 显示窗口，避免白屏闪烁。

---

有问题看父项目 `指令合集.txt`。
